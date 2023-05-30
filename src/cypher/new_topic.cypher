create (t: Topic {
    id:     $id,
    name:   $name,
    pages:  0,
    subs:   0,
    time:   $time
    desc:   $description
})
return t
// might add more fields. Stay tuned.
// Added info. 