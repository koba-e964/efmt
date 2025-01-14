% From: http://www1.erlang.org/examples/examples-2.0.html
connect(Host, User, Password) ->
    {ftp_server, Host} ! {connect,
                          self(),
                          User,
                          Password},
    receive
        {ftp_server, Reply} ->
            Reply;
        Other ->
            Other
    after
        10000 ->
            timeout
    end.

pwd(Handle) ->
    remote(Handle, pwd).
cd(Handle, Dir) ->
    remote(Handle, {cd, Dir}).
ls(Handle) ->
    remote(Handle, ls).
get(Handle, File) ->
    remote(Handle, {get, File}).
quit(Handle) ->
    remote(Handle, quit).

remote(Handle, Op) ->
    Handle ! {self(), Op},
    receive
        {ftp_server, Any} ->
            Any
    after
        1000 ->
            timeout
    end.

lcd(Dir) ->
    file:set_cwd(Dir),
    lpwd().
lpwd() ->
    cwd().
lls() ->
    element(2, file:list_dir(cwd())).

put(Handle, File) ->
    case file:read_file(File) of
        {ok, Contents} ->
            remote(Handle, {put, File, Contents});
        Other ->
            Other
    end.
