% From: http://www1.erlang.org/examples/examples-2.0.html

start() ->
    case (catch register(ftp_server,
                         spawn(?MODULE,
                               internal,
                               []))) of
        {'EXIT', _} ->
            already_started;
        Pid ->
            ok
    end.

internal() ->
    case file:consult("users") of
        {ok, Users} ->
            process_flag(trap_exit, true),
            loop(Users, 0);
        _ ->
            exit(no_users_allowed)
    end.

loop(Users, N) ->
    receive
        {connect, Pid, User, Password} ->
            io:format("connection request from:~p ~p ~p~n",
                      [Pid, User, Password]),
            case member({User, Password},
                        Users) of
                true ->
                    Max = max_connections(),
                    if
                        N > Max ->
                            Pid ! {ftp_server,
                                   {error,
                                    too_many_connections}},
                            loop(Users, N);
                        true ->
                            New =
                                spawn_link(?MODULE,
                                           handler,
                                           [Pid]),
                            Pid ! {ftp_server,
                                   {ok, New}},
                            loop(Users, N + 1)
                    end;
                false ->
                    Pid ! {ftp_server,
                           {error, rejected}},
                    loop(Users, N)
            end;
        {'EXIT', Pid} ->
            io:format("Handler ~p died~n", [Pid]),
            loop(Users, lists:max(N - 1, 0));
        Any ->
            io:format("received:~p~n", [Any]),
            loop(Users, N)
    end.

handler(Pid) ->
    receive
        {Pid, quit} ->
            Pid ! {ftp_server, ok};
        {Pid, Op} ->
            io:format("got:~p ~p~n", [Pid, Op]),
            Pid ! {ftp_server, do_op(Op)},
            handler(Pid)
    end.

do_op({cd, Dir}) ->
    file:set_cwd(Dir),
    cwd();
do_op(ls) ->
    element(2, file:list_dir(cwd()));
do_op(pwd) ->
    cwd();
do_op({get_file, File}) ->
    file:read_file(File).
