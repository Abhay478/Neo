match (s: Service {
        id: $id,
        by: $me
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
set p.by = null // service terminated
delete w

delete s