match (t: Topic {name: $tname}) 
create x = 
    (s: Service {id: $sid}) -[: serves {type: 'eh', service: s.id, topic: t.id}]-> (t) 
return x
// will add a type enum.