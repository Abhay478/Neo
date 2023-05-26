match (:Topic {id:$id}) <-[:follows]- (s: Account)
return s