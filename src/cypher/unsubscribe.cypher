match (me: Account {id: $me}) -[f: follows]-> (t: Topic {name: $tname})
with count(f) as c
with t.subs as q
delete f

// decrement subscriber count
set t.subs = q - 1
return 
    case 
        when c = 0 then true 
        else false
    end as out