match (t: Topic {name: $tname}) 
create x = 
    (s: Service {id: $sid}) -[: Serves {type: 'eh', service: s.id, topic: t.id}]-> (t) 
return x