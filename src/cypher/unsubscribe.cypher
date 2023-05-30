match (me: Account {id: $me}) -[f: follows]-> (t: Topic {name: $tname})
with count(f) as c
delete f
return c