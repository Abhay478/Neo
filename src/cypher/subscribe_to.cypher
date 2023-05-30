match (me: Account {id: $me}) 
match (this: Topic {name: $tname}) 
create x = 
    (me) -[out: follows {
        sub_id: $id, 
        from:   $from, 
        sub:    me.id, 
        topic:  this.id
    }]-> (this) 
return x