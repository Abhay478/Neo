create x = 
    (:Account {
        id:$obj,
        username:$unm, 
        password:$pswd, 
        disp_name: $dnm, 
        auth:'Subscriber'
    }) 
return x