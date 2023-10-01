_anek() {
    local i cur prev opts cmd
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
            anek,graph)
                cmd="anek__graph"
                ;;
            anek,help)
                cmd="anek__help"
                ;;
            anek,list)
                cmd="anek__list"
                ;;
            anek,new)
                cmd="anek__new"
                ;;
            anek,report)
                cmd="anek__report"
                ;;
            anek,run)
                cmd="anek__run"
                ;;
            anek,show)
                cmd="anek__show"
                ;;
            anek,variable)
                cmd="anek__variable"
                ;;
            anek,view)
                cmd="anek__view"
                ;;
            anek__help,completions)
                cmd="anek__help__completions"
                ;;
            anek__help,edit)
                cmd="anek__help__edit"
                ;;
            anek__help,graph)
                cmd="anek__help__graph"
                ;;
            anek__help,help)
                cmd="anek__help__help"
                ;;
            anek__help,list)
                cmd="anek__help__list"
                ;;
            anek__help,new)
                cmd="anek__help__new"
                ;;
            anek__help,report)
                cmd="anek__help__report"
                ;;
            anek__help,run)
                cmd="anek__help__run"
                ;;
            anek__help,show)
                cmd="anek__help__show"
                ;;
            anek__help,variable)
                cmd="anek__help__variable"
                ;;
            anek__help,view)
                cmd="anek__help__view"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        anek)
            opts="-q -h -V --quiet --help --version new variable list edit run completions report view show graph help"
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
            opts="-b -z -f -h --bash --zsh --fish --help"
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
            opts="-h --help <ANEK_FILE> [PATH]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "$(anek -q list)" -- "${cur}") )
            return 0
            ;;
        anek__graph)
            opts="-o -p -b -r -n -u -h --omit-orphans --pipelines --batches --raise-nodes --no-clusters --urls --help [PATH]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --pipelines)
                    COMPREPLY=($(compgen -W "$(anek -q list -p)" -- "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -W "$(anek -q list -p)" -- "${cur}"))
                    return 0
                    ;;
                --batches)
                    COMPREPLY=($(compgen -W "$(anek -q list -b)" -- "${cur}"))
                    return 0
                    ;;
                -b)
                    COMPREPLY=($(compgen -W "$(anek -q list -b)" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__help)
            opts="new variable list edit run completions report view show graph help"
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
        anek__help__graph)
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
        anek__help__new)
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
        anek__help__report)
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
        anek__help__show)
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
            COMPREPLY=( $(compgen -W "$(anek -q list)" -- "${cur}") )
            return 0
            ;;
        anek__help__variable)
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
        anek__help__view)
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
        anek__list)
            opts="-F -s -v -i -c -p -l -b -a -h --filter --search --variables --inputs --command --pipeline --loops --batch --all --help [PATH]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --filter)
                    COMPREPLY=($(compgen -W "$(anek -q list -v) $(anek -q list -c) $(anek -q list -i)" -- "${cur}"))
                    return 0
                    ;;
                -F)
                    COMPREPLY=($(compgen -W "$(anek -q list -v) $(anek -q list -c) $(anek -q list -i)" -- "${cur}"))
                    return 0
                    ;;
                --search)
                    COMPREPLY=($(compgen -W "$(anek -q list -v) $(anek -q list -c) $(anek -q list -i)" -- "${cur}"))
                    return 0
                    ;;
                -s)
                    COMPREPLY=($(compgen -W "$(anek -q list -v) $(anek -q list -c) $(anek -q list -i)" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__new)
            opts="-v -h --variables --help [PATH]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --variables)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__report)
            opts="-f -h --filename --help [PATH]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --filename)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -f "${cur}"))
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
            opts="-c -C -p -r -R -e -E -s -b -l -i -P -d -o -h --command --command-template --pipeline --render --render-template --export --export-format --select-inputs --batch --loop --input --pipable --demo --overwrite --help [PATH] [COMMAND_ARGS]..."
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
                --render)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -r)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --render-template)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -R)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --export)
                    COMPREPLY=($(compgen -W "$(anek -q list -v)" -- "${cur}"))
                    return 0
                    ;;
                -e)
                    COMPREPLY=($(compgen -W "$(anek -q list -v)" -- "${cur}"))
                    return 0
                    ;;
                --export-format)
                    COMPREPLY=($(compgen -W "csv json plain" -- "${cur}"))
                    return 0
                    ;;
                -E)
                    COMPREPLY=($(compgen -W "csv json plain" -- "${cur}"))
                    return 0
                    ;;
                --select-inputs)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -s)
                    COMPREPLY=($(compgen -f "${cur}"))
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
                --input)
                    COMPREPLY=($(compgen -W "$(anek -q list -i)" -- "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -W "$(anek -q list -i)" -- "${cur}"))
                    return 0
                    ;;
                --overwrite)
                    COMPREPLY=($(compgen -S ":" -W "$(anek -q list -v)" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -S ":" -W "$(anek -q list -v)" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__show)
            opts="-h --help <ANEK_FILE> [PATH]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=($(compgen -W "$(anek -q list)" -- "${cur}"))
            return 0
            ;;
        anek__variable)
            opts="-s -S -a -l -d -i -u -h --scan-inputs --scan-commands --add --list --details --info --update --help [PATH]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --info)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --update)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        anek__view)
            opts="-h --help [PATH]"
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
    esac
}

complete -F _anek -o bashdefault -o default anek
