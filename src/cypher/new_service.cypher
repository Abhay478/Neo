match (t: Topic {name: $tname}) 
match (me: Account {id: $me}) // is a service provider or admin, checked in server.
create x = 
    (me) -[: provides]-> (s: Service {
        id: $sid, 
        by: $me
        typ: $typ
    }) -[: serves {
        serv:   s.id, 
        topic:  t.id
    }]-> (t) 
return x