match (me: Account {id: $me}) -[f: follows]-> (t: Topic {name: $tname})
return count(f) as c