match (t: Topic {name: $tname})
create x = 
    (p: Page {
        id:     $pid,
        title:  $title,
        body:   $body,
        time:   $time,
        by:     $sid
    }) <-[: contains]- (t)
return x