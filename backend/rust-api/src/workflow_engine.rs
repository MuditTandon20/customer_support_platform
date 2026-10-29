//! Request-bound automation. Effects and run records share the caller's transaction.
//! No timer, detached task, or external write is permitted on the sleeping-host profile.
use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::TicketStatus,
    policy::{self, Definition},
};
use serde_json::{json, Value};
use sqlx::{PgConnection, Row};
use uuid::Uuid;

pub async fn execute(
    c: &mut PgConnection,
    a: &AuthUser,
    ticket: Uuid,
    d: &Definition,
    simulation: bool,
) -> AppResult<(String, Vec<Value>)> {
    policy::validate(d)?;
    sqlx::query("SAVEPOINT workflow_effects")
        .execute(&mut *c)
        .await?;
    let (mut status, mut priority): (TicketStatus, String) = sqlx::query_as(
        "SELECT status,priority FROM tickets WHERE organization_id=$1 AND id=$2 FOR UPDATE",
    )
    .bind(a.org_id)
    .bind(ticket)
    .fetch_one(&mut *c)
    .await?;
    let mut next = Some(d.nodes[0].id.clone());
    let mut trace = Vec::new();
    let mut state = "completed";
    for _ in 0..32 {
        let Some(id) = next.take() else { break };
        let n = d.nodes.iter().find(|n| n.id == id).unwrap();
        trace.push(json!({"node":n.id,"kind":n.kind,"value":n.value,"simulation":simulation}));
        next = n.next.clone();
        let step:AppResult<()>=async {
            match n.kind.as_str() {
                "condition"=>{if priority!=n.value {next=n.otherwise.clone();}},
                "approval"=>{state="awaiting_human";next=None;
                    if !simulation {
                        let request:Uuid=sqlx::query_scalar("INSERT INTO tool_executions(organization_id,ticket_id,requested_by,action,arguments,state,idempotency_key) VALUES($1,$2,$3,'human.review',$4,'pending',$5) RETURNING id").bind(a.org_id).bind(ticket).bind(a.user_id).bind(json!({"reason":if n.value.is_empty(){"Workflow checkpoint"}else{&n.value}})).bind(Uuid::new_v4()).fetch_one(&mut *c).await?;
                        trace.push(json!({"node":n.id,"approval_id":request,"kind":"approval_request","value":"Human handoff; no automatic continuation"}));
                    }
                },
                "fallback"=>{state="escalated";next=None;},
                "action"=>{
                    let (action,args)=if let Some(v)=n.value.strip_prefix("label:") {("ticket.label",json!({"label":v}))}
                    else if let Some(v)=n.value.strip_prefix("status:") {
                        let to:TicketStatus=serde_json::from_value(json!(v))?;
                        if !policy::transition(status,to){return Err(AppError::BadRequest("Workflow would violate ticket state machine".into()));}
                        status=to;("ticket.status",json!({"status":v}))
                    }else if let Some(v)=n.value.strip_prefix("priority:"){priority=v.into();("ticket.priority",json!({"priority":v}))}
                    else if n.value=="notify:agent" {("agent.notify",json!({}))}
                    else if n.value=="csat:request" {("csat.request",json!({}))}
                    else{return Err(AppError::BadRequest("Unsupported local action".into()));};
                    if !simulation {crate::handlers::platform::execute_local(c,a,ticket,action,&args).await?;}
                },_=>{},
            }Ok(())
        }.await;
        if let Err(error) = step {
            let reason = match error {
                AppError::BadRequest(m) | AppError::Forbidden(m) | AppError::NotFound(m) => m,
                _ => "Action failed; inspect server logs".into(),
            };
            trace.push(json!({"node":n.id,"kind":"error","value":reason,"rolled_back":true}));
            state = "failed";
            sqlx::query("ROLLBACK TO SAVEPOINT workflow_effects")
                .execute(&mut *c)
                .await?;
            break;
        }
    }
    sqlx::query("RELEASE SAVEPOINT workflow_effects")
        .execute(&mut *c)
        .await?;
    Ok((state.into(), trace))
}

pub async fn dispatch(
    c: &mut PgConnection,
    a: &AuthUser,
    event: &str,
    ticket: Uuid,
) -> AppResult<()> {
    // Called synchronously before commit. Restarts cannot lose a detached execution.
    let rows=sqlx::query("SELECT id,version,published FROM workflows WHERE organization_id=$1 AND published IS NOT NULL AND published->'nodes'->0->>'value'=$2 ORDER BY id LIMIT 25").bind(a.org_id).bind(event).fetch_all(&mut *c).await?;
    for row in rows {
        let count:i64=sqlx::query_scalar("SELECT count(*) FROM workflow_runs WHERE organization_id=$1 AND created_at>date_trunc('day',now())").bind(a.org_id).fetch_one(&mut *c).await?;
        let id: Uuid = row.get("id");
        let (state, trace) = if count >= 500 {
            (
                "quota_limited".into(),
                vec![
                    json!({"kind":"error","node":"quota","value":"Daily execution quota reached; continue support manually"}),
                ],
            )
        } else {
            execute(
                c,
                a,
                ticket,
                &serde_json::from_value(row.get::<Value, _>("published"))?,
                false,
            )
            .await?
        };
        sqlx::query("INSERT INTO workflow_runs(organization_id,workflow_id,ticket_id,version,simulation,state,trace,idempotency_key,trigger_event) VALUES($1,$2,$3,$4,false,$5,$6,$7,$8)").bind(a.org_id).bind(id).bind(ticket).bind(row.get::<i32,_>("version")).bind(&state).bind(json!(trace)).bind(Uuid::new_v4()).bind(event).execute(&mut *c).await?;
        crate::handlers::platform::audit(
            c,
            a,
            "workflow.triggered",
            id,
            json!({"ticket_id":ticket,"trigger":event,"state":state}),
        )
        .await?;
        if state == "quota_limited" {
            break;
        }
    }
    Ok(())
}
