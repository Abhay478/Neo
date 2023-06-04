match (a: Account {id: $me}) -[: provides]-> (s: Service)
return s