match (me: Account {id: $me}) -[f: follows]-> (t: Topic {name: $tname})
return 
    case
        when count(f) = 0 then false
        else true
    end as out