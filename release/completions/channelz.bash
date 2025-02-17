_basher___channelz() {
	local cur prev opts
	COMPREPLY=()
	cur="${COMP_WORDS[COMP_CWORD]}"
	prev="${COMP_WORDS[COMP_CWORD-1]}"
	opts=()
	[[ " ${COMP_LINE} " =~ " --clean " ]] || opts+=("--clean")
	[[ " ${COMP_LINE} " =~ " --clean-only " ]] || opts+=("--clean-only")
	[[ " ${COMP_LINE} " =~ " --force " ]] || opts+=("--force")
	if [[ ! " ${COMP_LINE} " =~ " -h " ]] && [[ ! " ${COMP_LINE} " =~ " --help " ]]; then
		opts+=("-h")
		opts+=("--help")
	fi
	[[ " ${COMP_LINE} " =~ " --no-br " ]] || opts+=("--no-br")
	[[ " ${COMP_LINE} " =~ " --no-gz " ]] || opts+=("--no-gz")
	if [[ ! " ${COMP_LINE} " =~ " -p " ]] && [[ ! " ${COMP_LINE} " =~ " --progress " ]]; then
		opts+=("-p")
		opts+=("--progress")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -V " ]] && [[ ! " ${COMP_LINE} " =~ " --version " ]]; then
		opts+=("-V")
		opts+=("--version")
	fi
	if [[ ! " ${COMP_LINE} " =~ " -l " ]] && [[ ! " ${COMP_LINE} " =~ " --list " ]]; then
		opts+=("-l")
		opts+=("--list")
	fi
	opts=" ${opts[@]} "
	if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
		COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
		return 0
	fi
	case "${prev}" in
		--list|-l)
			if [ -z "$( declare -f _filedir )" ]; then
				COMPREPLY=( $( compgen -f "${cur}" ) )
			else
				COMPREPLY=( $( _filedir ) )
			fi
			return 0
			;;
		*)
			COMPREPLY=()
			;;
	esac
	COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
	return 0
}
complete -F _basher___channelz -o bashdefault -o default channelz
