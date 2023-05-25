match (me: Account {id: $me}) -[f: follows]-> (t: Topic {name: $tname})
delete f