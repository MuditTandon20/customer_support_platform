use crate::{
    error::{AppError, AppResult},
    models::TicketStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub fn transition(from: TicketStatus, to: TicketStatus) -> bool {
    use TicketStatus::*;
    from == to
        || matches!(
            (from, to),
            (New, Open | Pending | OnHold | Solved)
                | (Open, Pending | OnHold | Solved)
                | (Pending, Open | OnHold | Solved)
                | (OnHold, Open | Pending | Solved)
                | (Solved, Open | Closed)
        )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub otherwise: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub nodes: Vec<Node>,
}

pub fn validate(d: &Definition) -> AppResult<()> {
    let bad = |m: &str| AppError::BadRequest(m.into());
    if d.nodes.is_empty() || d.nodes.len() > 32 {
        return Err(bad("Workflow needs 1–32 nodes"));
    }
    let map: HashMap<_, _> = d.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    if map.len() != d.nodes.len() || d.nodes.iter().any(|n| n.id.is_empty() || n.id.len() > 80) {
        return Err(bad("Node IDs must be unique and nonempty"));
    }
    if d.nodes[0].kind != "trigger" || d.nodes.iter().filter(|n| n.kind == "trigger").count() != 1 {
        return Err(bad("First node must be the only trigger"));
    }
    for n in &d.nodes {
        match n.kind.as_str() {
            "trigger"
                if [
                    "",
                    "manual",
                    "ticket.created",
                    "customer.replied",
                    "ticket.resolved",
                    "ai.abstained",
                ]
                .contains(&n.value.as_str()) => {}
            "approval" | "fallback" => {}
            "condition"
                if ["low", "normal", "high", "urgent"].contains(&n.value.as_str())
                    && n.next.is_some()
                    && n.otherwise.is_some() => {}
            "action"
                if n.value
                    .strip_prefix("label:")
                    .is_some_and(|v| !v.trim().is_empty() && v.len() <= 40) => {}
            "action"
                if n.value.strip_prefix("status:").is_some_and(|v| {
                    ["open", "pending", "on_hold", "solved", "closed"].contains(&v)
                }) => {}
            "action"
                if n.value
                    .strip_prefix("priority:")
                    .is_some_and(|v| ["low", "normal", "high", "urgent"].contains(&v)) => {}
            "action" if ["notify:agent", "csat:request"].contains(&n.value.as_str()) => {}
            "delay" => return Err(bad("Durable delays are not enabled; use an approval pause")),
            _ => return Err(bad("Unsupported node or invalid value")),
        }
        if n.kind != "condition" && n.otherwise.is_some() {
            return Err(bad("Only conditions may branch"));
        }
        for edge in [&n.next, &n.otherwise].into_iter().flatten() {
            if !map.contains_key(edge.as_str()) {
                return Err(bad("Dangling edge"));
            }
        }
    }
    fn visit<'a>(
        id: &'a str,
        map: &HashMap<&'a str, &'a Node>,
        active: &mut HashSet<&'a str>,
        done: &mut HashSet<&'a str>,
    ) -> bool {
        if active.contains(id) {
            return false;
        }
        if done.contains(id) {
            return true;
        }
        active.insert(id);
        let node = map[id];
        for edge in [&node.next, &node.otherwise].into_iter().flatten() {
            if !visit(edge, map, active, done) {
                return false;
            }
        }
        active.remove(id);
        done.insert(id);
        true
    }
    let mut done = HashSet::new();
    if !visit(&d.nodes[0].id, &map, &mut HashSet::new(), &mut done) {
        return Err(bad("Cycles are forbidden"));
    }
    if done.len() != d.nodes.len() {
        return Err(bad("Every node must be reachable"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_is_terminal() {
        assert!(!transition(TicketStatus::Closed, TicketStatus::Open));
        assert!(transition(TicketStatus::Solved, TicketStatus::Open));
    }
    #[test]
    fn graph_rejects_cycles_and_unknown_actions() {
        let mut d = Definition {
            nodes: vec![Node {
                id: "start".into(),
                kind: "trigger".into(),
                value: "".into(),
                next: Some("start".into()),
                otherwise: None,
            }],
        };
        assert!(validate(&d).is_err());
        d.nodes[0].next = None;
        assert!(validate(&d).is_ok());
        d.nodes.push(Node {
            id: "pay".into(),
            kind: "action".into(),
            value: "refund:10".into(),
            next: None,
            otherwise: None,
        });
        d.nodes[0].next = Some("pay".into());
        assert!(validate(&d).is_err());
    }
}
