_anek() {
    local i cur prev opts cmds
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="anek"
                ;;
            anek,completions)
                cmd="anek__completions"
                ;;
            anek,edit)
                cmd="anek__edit"
                ;;
            anek,help)
                cmd="anek__help"
                ;;
            anek,input)
                cmd="anek__input"
                ;;
            anek,list)
                cmd="anek__list"
                ;;
            anek,run)
                cmd="anek__run"
                ;;
            anek__help,completions)
                cmd="anek__help__completions"
                ;;
            anek__help,edit)
                cmd="anek__help__edit"
                ;;
            anek__help,help)
                cmd="anek__help__help"
                ;;
            anek__help,input)
                cmd="anek__help__input"
                ;;
            anek__help,list)
                cmd="anek__help__list"
                ;;
            anek__help,run)
                cmd="anek__help__run"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        anek)
            opts="-q -h -V --quiet --help --version input list edit run completions help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__completions)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__edit)
            opts="-e -h --echo --help <ANEK_FILE> [PATH]"
            if [[ ${cur} == -* ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
		    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "$(anek -q list -a)" -- "${cur}") )
            return 0
            ;;
        anek__help)
            opts="input list edit run completions help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__help__completions)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__help__edit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__help__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__help__input)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__help__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__help__run)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__input)
            opts="-l -d -i -h --list --details --info --help [PATH]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --info)
                    COMPREPLY=($(compgen -W "$(anek -q list -i)" -- "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -W "$(anek -q list -i)" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__list)
            opts="-F -a -i -f -c -p -l -b -h --filter --all --input --favorite --command --pipeline --loops --batch --help [PATH]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --filter)
                    COMPREPLY=($(compgen -W "$(anek -q list -i)" -- "${cur}"))
                    return 0
                    ;;
                -F)
                    COMPREPLY=($(compgen -W "$(anek -q list -i)" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__run)
            opts="-c -C -p -b -l -f -P -d -o -h --command --command-template --pipeline --batch --loop --favorite --pipable --demo --overwrite --help [PATH]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --command)
                    COMPREPLY=($(compgen -W "$(anek -q list -c)" -- "${cur}"))
                    return 0
                    ;;
                -c)
                    COMPREPLY=($(compgen -W "$(anek -q list -c)" -- "${cur}"))
                    return 0
                    ;;
                --command-template)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pipeline)
                    COMPREPLY=($(compgen -W "$(anek -q list -p)" -- "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -W "$(anek -q list -p)" -- "${cur}"))
                    return 0
                    ;;
                --batch)
                    COMPREPLY=($(compgen -W "$(anek -q list -b)" -- "${cur}"))
                    return 0
                    ;;
                -b)
                    COMPREPLY=($(compgen -W "$(anek -q list -b)" -- "${cur}"))
                    return 0
                    ;;
                --loop)
                    COMPREPLY=($(compgen -W "$(anek -q list -l)" -- "${cur}"))
                    return 0
                    ;;
                -l)
                    COMPREPLY=($(compgen -W "$(anek -q list -l)" -- "${cur}"))
                    return 0
                    ;;
                --favorite)
                    COMPREPLY=($(compgen -W "$(anek -q list -f)" -- "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -W "$(anek -q list -f)" -- "${cur}"))
                    return 0
                    ;;
                --overwrite)
                    COMPREPLY=($(compgen -S ":" -W "$(anek -q list -i)" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -S ":" -W "$(anek -q list -i)" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

complete -F _anek -o bashdefault -o default anek
