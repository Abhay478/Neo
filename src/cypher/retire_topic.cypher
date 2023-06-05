match (t: Topic {
        id: $id, 
        owner: $me
    })
with count(t) as c

// Find Services
match (s: Service) -[r]-> (t)
delete r // Unhook services

// Find pages
with t
match (t) -[u]-> (p: Page)
delete u // Unhook pages

delete p // Delete pages
delete s // Delete services

delete t // Delete topic

// returns if topic exists: If you don't own the topic, none of the above will happen
return case
    when c == 0 then false
    else true
end as out