match (s: Service {
        id: $id, // Already pretty secure, since only the provider has the id.
        by: $me // Ensures that it is indeed your service. If it isn't, silently fails.
    })
with count(s) as c
// Unhook from topic
match (s) -[r: serves]-> ()
delete r

// Unhook pages.
with s
match (s) -[w: wrote]-> (p: Page)
set p.by = null // service terminated, pages just floating around. Will die with topic
delete w

// Unhook SP
with s
match (s) <-[r: provides]- (a: Account)
delete r

delete s

return case
    when c == 0 then false
    else true
end as out