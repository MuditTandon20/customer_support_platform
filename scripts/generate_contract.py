"""Generate the endpoint inventory from Rust routing; explicitly document core request schemas."""
import json
import pathlib
import re
root=pathlib.Path(__file__).parents[1]
source=(root/'backend/rust-api/src/main.rs').read_text(encoding='utf-8')
schema={
 'CreateTicket':{'type':'object','required':['subject'],'properties':{'subject':{'type':'string','minLength':1,'maxLength':200},'description':{'type':'string','maxLength':20000}}},
 'Reply':{'type':'object','additionalProperties':False,'required':['body','idempotency_key'],'properties':{'body':{'type':'string','minLength':1,'maxLength':20000},'internal':{'type':'boolean','default':False},'idempotency_key':{'type':'string','format':'uuid'},'proposal_id':{'type':['string','null'],'format':'uuid'}}},
 'Tool':{'type':'object','additionalProperties':False,'required':['ticket_id','action','arguments','idempotency_key'],'properties':{'ticket_id':{'type':'string','format':'uuid'},'action':{'type':'string','enum':['ticket.label','ticket.status','account.lookup','refund','subscription.cancel','data.export']},'arguments':{'type':'object','description':'label:string, status:ticket_status, empty lookup object, or reason:string for sensitive requests. Extra fields rejected.'},'idempotency_key':{'type':'string','format':'uuid'}}},
 'Node':{'type':'object','additionalProperties':False,'required':['id','kind'],'properties':{'id':{'type':'string'},'kind':{'type':'string','enum':['trigger','condition','action','approval','fallback']},'value':{'type':'string'},'next':{'type':['string','null']},'otherwise':{'type':['string','null']}}},
 'Workflow':{'type':'object','required':['name','definition'],'properties':{'name':{'type':'string','maxLength':160},'definition':{'type':'object','required':['nodes'],'properties':{'nodes':{'type':'array','minItems':1,'maxItems':32,'items':{'$ref':'#/components/schemas/Node'}}}}}},
 'Run':{'type':'object','required':['ticket_id','simulation','idempotency_key'],'properties':{'ticket_id':{'type':'string','format':'uuid'},'simulation':{'type':'boolean'},'idempotency_key':{'type':'string','format':'uuid'}}},
 'Knowledge':{'type':'object','required':['title','content','visibility'],'properties':{'title':{'type':'string','maxLength':200},'content':{'type':'string','maxLength':100000},'visibility':{'type':'string','enum':['team','public']},'expires_at':{'type':['string','null'],'format':'date-time'}}},
}
paths={}
for path,body in re.findall(r'\.route\(\s*"([^"]+)"\s*,(.*?)\)\s*(?=\.route|\.layer)',source,re.S):
    normalized=re.sub(r':(\w+)',r'{\1}',path)
    for method in re.findall(r'\b(get|post|patch|delete)\(',body):
        public=path.startswith(('/auth/','/public/')) or path in ['/health','/ready','/live','/release','/openapi.json']
        operation={'summary':f'{method.upper()} {normalized}','security':[] if public else [{'bearer':[]},{'sessionCookie':[]}],'responses':{'200':{'description':'Success; see API guide for response semantics'},'400':{'description':'Validation/policy failure'},'401':{'description':'Authentication required'},'403':{'description':'Role or origin denied'},'404':{'description':'Absent or inaccessible resource; tenant existence not disclosed'},'429':{'description':'Rate limit reached'}}}
        params=[{'name':p,'in':'path','required':True,'schema':{'type':'string','format':'uuid'} if p=='id' else {'type':'string'}} for p in re.findall(r'{(\w+)}',normalized)]
        if params:operation['parameters']=params
        model='Reply' if path.endswith('/messages') else 'Tool' if path in ['/tools','/tools/preview'] else 'Run' if path.endswith('/run') else 'Workflow' if path=='/workflows' or (path=='/workflows/:id' and method=='patch') else 'Knowledge' if path=='/knowledge' else 'CreateTicket' if path=='/tickets' else None
        if model and method in ['post','patch']:operation['requestBody']={'required':True,'content':{'application/json':{'schema':{'$ref':f'#/components/schemas/{model}'}}}}
        paths.setdefault(normalized,{})[method]=operation
document={'openapi':'3.1.0','info':{'title':'Aster Support OS core API','version':'0.2.0','description':'Generated endpoint inventory with core request schemas. Existing unversioned paths retained for compatibility. Full response schema coverage remains incomplete.'},'servers':[{'url':'http://localhost:8080'}],'components':{'securitySchemes':{'bearer':{'type':'http','scheme':'bearer','bearerFormat':'JWT'},'sessionCookie':{'type':'apiKey','in':'cookie','name':'aster_session'}},'schemas':schema},'paths':paths}
(root/'backend/rust-api/openapi.json').write_text(json.dumps(document,indent=2)+'\n',encoding='utf-8')
print(f'Generated inventory for {len(paths)} paths')
