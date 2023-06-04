match (s: Service {
        id: $id,
        by: $me // Ensures that it is indeed your service. If it isn't, silently fails.
    })

// Unhook from topic
match (s) -[r: serves]-> ()
delete r

// // Unhook and delete pages - might not be necessary, in which case set p.by = null
// match (s) -[w: wrote]-> (p: Page) <-[c: contains]- (t)
// delete c
// delete w
// delete p

// Unhook pages.
match (s) -[w: wrote]-> (p: Page)
set p.by = null // service terminated, pages just floating around. Will die with topic
delete w

match (s) <-[r: provides]- (a: Account)
delete r

delete s