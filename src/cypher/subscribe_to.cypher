match (me: Account {id: $me}) 
match (this: Topic {name: $this}) 
create x = 
    (me) -[out: follows {
        subs_id: $id, 
        from: $from, 
        sub: me.id, 
        topic: this.id
    }]-> (this) 
return x