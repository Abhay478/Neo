create (t: Topic {
    id:     $id,
    name:   $name,
    pages:  0,
    subs:   0, // subscriber count
    time:   $time, // of creation
    desc:   $description,
    owner:  $me
})
return t
// might add more fields. Stay tuned.
// Added info. 