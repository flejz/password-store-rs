use anyhow::{bail, Result};

pub fn run(shell: &str) -> Result<()> {
    match shell.to_ascii_lowercase().as_str() {
        "bash" => print!("{}", BASH),
        "zsh" => print!("{}", ZSH),
        "fish" => print!("{}", FISH),
        "powershell" | "ps" | "ps1" => print!("{}", POWERSHELL),
        "elvish" => print!("{}", ELVISH),
        other => bail!(
            "Unknown shell: {}. Supported: bash, zsh, fish, powershell, elvish",
            other
        ),
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Bash
// ---------------------------------------------------------------------------
const BASH: &str = r#"
_pass() {
    local cur prev words cword
    _init_completion 2>/dev/null || {
        COMPREPLY=()
        cur="${COMP_WORDS[COMP_CWORD]}"
        prev="${COMP_WORDS[COMP_CWORD-1]}"
        words=("${COMP_WORDS[@]}")
        cword=$COMP_CWORD
    }

    local store="${PASSWORD_STORE_DIR:-$HOME/.password-store}"

    __pass_names() {
        find "$store" -name "*.gpg" 2>/dev/null \
            | sed "s|${store}/||; s|\.gpg$||" \
            | sort
    }

    __pass_dirs() {
        find "$store" -mindepth 1 -type d 2>/dev/null \
            | sed "s|${store}/||" \
            | sort
    }

    local subcmds="init ls list show insert add generate rm delete remove find search grep edit cp copy mv rename git completion help"

    if [[ $cword -eq 1 ]]; then
        # Offer subcommands AND password names — `pass <tab>` acts like `pass show <tab>`
        COMPREPLY=($(compgen -W "$subcmds $(__pass_names)" -- "$cur"))
        return
    fi

    local cmd="${words[1]}"
    case "$cmd" in
        show|edit)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=($(compgen -W "--clip --line" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "$(__pass_names)" -- "$cur"))
            fi
            ;;
        insert|add)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=($(compgen -W "--echo --multiline --force" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "$(__pass_names)" -- "$cur"))
            fi
            ;;
        generate)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=($(compgen -W "--no-symbols --clip --in-place --force" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "$(__pass_names)" -- "$cur"))
            fi
            ;;
        rm|delete|remove)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=($(compgen -W "--recursive --force" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "$(__pass_names)" -- "$cur"))
            fi
            ;;
        cp|copy|mv|rename)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=($(compgen -W "--force" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "$(__pass_names)" -- "$cur"))
            fi
            ;;
        ls|list)
            COMPREPLY=($(compgen -W "$(__pass_dirs)" -- "$cur"))
            ;;
        find|search)
            COMPREPLY=($(compgen -W "$(__pass_names)" -- "$cur"))
            ;;
        init)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=($(compgen -W "--path" -- "$cur"))
            fi
            ;;
        completion)
            COMPREPLY=($(compgen -W "bash zsh fish powershell elvish" -- "$cur"))
            ;;
    esac
}

complete -F _pass pass
"#;

// ---------------------------------------------------------------------------
// Zsh
// ---------------------------------------------------------------------------
const ZSH: &str = r#"
#compdef pass

_pass() {
    local state

    _arguments \
        '1: :->command' \
        '*: :->args'

    local store="${PASSWORD_STORE_DIR:-$HOME/.password-store}"

    __pass_names() {
        find "$store" -name "*.gpg" 2>/dev/null \
            | sed "s|${store}/||; s|\.gpg$||" \
            | sort
    }

    __pass_dirs() {
        find "$store" -mindepth 1 -type d 2>/dev/null \
            | sed "s|${store}/||" \
            | sort
    }

    case $state in
        command)
            # Subcommands + password names so `pass <tab>` also acts as show
            local commands=(
                'init:Initialize the password store'
                'ls:List passwords'
                'list:List passwords'
                'show:Show a password'
                'insert:Insert a password'
                'add:Insert a password'
                'generate:Generate a password'
                'rm:Remove a password'
                'delete:Remove a password'
                'remove:Remove a password'
                'find:Search password names'
                'search:Search password names'
                'grep:Search inside passwords'
                'edit:Edit a password'
                'cp:Copy a password'
                'copy:Copy a password'
                'mv:Move a password'
                'rename:Move a password'
                'git:Run git in the store'
                'completion:Generate shell completion script'
                'help:Print help'
            )
            _alternative \
                'commands:command:(('"${commands[@]}"'))' \
                'passwords:password:($(__pass_names))'
            ;;
        args)
            case "${words[2]}" in
                show)
                    _arguments \
                        '(-c --clip)'{-c,--clip}'[Copy to clipboard]' \
                        '(-n --line)'{-n,--line}'[Line number]:line:' \
                        ':password name:($(__pass_names))'
                    ;;
                insert|add)
                    _arguments \
                        '(-e --echo)'{-e,--echo}'[Echo input]' \
                        '(-m --multiline)'{-m,--multiline}'[Multiline input]' \
                        '(-f --force)'{-f,--force}'[Overwrite without confirm]' \
                        ':password name:($(__pass_names))'
                    ;;
                generate)
                    _arguments \
                        '(-n --no-symbols)'{-n,--no-symbols}'[No symbols]' \
                        '(-c --clip)'{-c,--clip}'[Copy to clipboard]' \
                        '(-i --in-place)'{-i,--in-place}'[Replace first line only]' \
                        '(-f --force)'{-f,--force}'[Overwrite without confirm]' \
                        ':password name:($(__pass_names))' \
                        '::length:'
                    ;;
                rm|delete|remove)
                    _arguments \
                        '(-r --recursive)'{-r,--recursive}'[Recursive]' \
                        '(-f --force)'{-f,--force}'[Force]' \
                        ':password name:($(__pass_names))'
                    ;;
                edit)
                    _arguments ':password name:($(__pass_names))'
                    ;;
                cp|copy|mv|rename)
                    _arguments \
                        '(-f --force)'{-f,--force}'[Overwrite without confirm]' \
                        ':source:($(__pass_names))' \
                        ':destination:($(__pass_names))'
                    ;;
                find|search)
                    _arguments '*:pattern:'
                    ;;
                ls|list)
                    _arguments ':subfolder:($(__pass_dirs))'
                    ;;
                init)
                    _arguments \
                        '(-p --path)'{-p,--path}'[Subfolder path]:path:_directories' \
                        '*:GPG key ID:'
                    ;;
                completion)
                    _arguments ':shell:(bash zsh fish powershell elvish)'
                    ;;
            esac
            ;;
    esac
}

_pass
"#;

// ---------------------------------------------------------------------------
// Fish
// ---------------------------------------------------------------------------
const FISH: &str = r#"
function __pass_names
    set store (if set -q PASSWORD_STORE_DIR; echo $PASSWORD_STORE_DIR; else; echo $HOME/.password-store; end)
    find $store -name "*.gpg" 2>/dev/null | sed "s|$store/||; s|\.gpg\$||" | sort
end

function __pass_dirs
    set store (if set -q PASSWORD_STORE_DIR; echo $PASSWORD_STORE_DIR; else; echo $HOME/.password-store; end)
    find $store -mindepth 1 -type d 2>/dev/null | sed "s|$store/||" | sort
end

function __pass_needs_command
    set cmd (commandline -opc)
    test (count $cmd) -eq 1
end

function __pass_using_command
    set cmd (commandline -opc)
    test (count $cmd) -gt 1; and test $cmd[2] = $argv[1]
end

# At position 1: subcommands AND password names (pass <tab> = pass show <tab>)
complete -c pass -f -n __pass_needs_command -a init              -d "Initialize the password store"
complete -c pass -f -n __pass_needs_command -a "ls list"         -d "List passwords"
complete -c pass -f -n __pass_needs_command -a show              -d "Show a password"
complete -c pass -f -n __pass_needs_command -a "insert add"      -d "Insert a password"
complete -c pass -f -n __pass_needs_command -a generate          -d "Generate a password"
complete -c pass -f -n __pass_needs_command -a "rm delete remove" -d "Remove a password"
complete -c pass -f -n __pass_needs_command -a "find search"     -d "Search password names"
complete -c pass -f -n __pass_needs_command -a grep              -d "Search inside passwords"
complete -c pass -f -n __pass_needs_command -a edit              -d "Edit a password"
complete -c pass -f -n __pass_needs_command -a "cp copy"         -d "Copy a password"
complete -c pass -f -n __pass_needs_command -a "mv rename"       -d "Move a password"
complete -c pass -f -n __pass_needs_command -a git               -d "Run git in the store"
complete -c pass -f -n __pass_needs_command -a completion        -d "Generate shell completion script"
complete -c pass -f -n __pass_needs_command -a "(__pass_names)"  -d "Show password"

# show
complete -c pass -f -n "__pass_using_command show" -a "(__pass_names)"
complete -c pass -f -n "__pass_using_command show" -l clip -s c -d "Copy to clipboard"
complete -c pass -f -n "__pass_using_command show" -l line -s n -d "Line number"

# insert / add
for __subcmd in insert add
    complete -c pass -f -n "__pass_using_command $__subcmd" -a "(__pass_names)"
    complete -c pass -f -n "__pass_using_command $__subcmd" -l echo      -s e -d "Echo input"
    complete -c pass -f -n "__pass_using_command $__subcmd" -l multiline -s m -d "Multiline input"
    complete -c pass -f -n "__pass_using_command $__subcmd" -l force     -s f -d "Overwrite without confirm"
end

# generate
complete -c pass -f -n "__pass_using_command generate" -a "(__pass_names)"
complete -c pass -f -n "__pass_using_command generate" -l no-symbols -s n -d "Alphanumeric only"
complete -c pass -f -n "__pass_using_command generate" -l clip       -s c -d "Copy to clipboard"
complete -c pass -f -n "__pass_using_command generate" -l in-place   -s i -d "Replace first line only"
complete -c pass -f -n "__pass_using_command generate" -l force      -s f -d "Overwrite without confirm"

# rm / delete / remove
for __subcmd in rm delete remove
    complete -c pass -f -n "__pass_using_command $__subcmd" -a "(__pass_names)"
    complete -c pass -f -n "__pass_using_command $__subcmd" -l recursive -s r -d "Recursive"
    complete -c pass -f -n "__pass_using_command $__subcmd" -l force     -s f -d "Force"
end

# ls / list
for __subcmd in ls list
    complete -c pass -f -n "__pass_using_command $__subcmd" -a "(__pass_dirs)"
end

# edit
complete -c pass -f -n "__pass_using_command edit" -a "(__pass_names)"

# cp / copy
for __subcmd in cp copy
    complete -c pass -f -n "__pass_using_command $__subcmd" -a "(__pass_names)"
    complete -c pass -f -n "__pass_using_command $__subcmd" -l force -s f -d "Overwrite without confirm"
end

# mv / rename
for __subcmd in mv rename
    complete -c pass -f -n "__pass_using_command $__subcmd" -a "(__pass_names)"
    complete -c pass -f -n "__pass_using_command $__subcmd" -l force -s f -d "Overwrite without confirm"
end

# find / search (complete with names as hints)
for __subcmd in find search
    complete -c pass -f -n "__pass_using_command $__subcmd" -a "(__pass_names)"
end

# completion
complete -c pass -f -n "__pass_using_command completion" -a "bash zsh fish powershell elvish"
"#;

// ---------------------------------------------------------------------------
// PowerShell
// ---------------------------------------------------------------------------
const POWERSHELL: &str = r#"
Register-ArgumentCompleter -Native -CommandName pass -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $store = if ($env:PASSWORD_STORE_DIR) { $env:PASSWORD_STORE_DIR } else { "$HOME\.password-store" }
    $elements = $commandAst.CommandElements

    function Get-PassNames {
        if (Test-Path $store) {
            Get-ChildItem -Path $store -Filter "*.gpg" -Recurse -ErrorAction SilentlyContinue |
                ForEach-Object {
                    $_.FullName.Substring($store.Length + 1) -replace '\.gpg$','' -replace '\\','/'
                }
        }
    }

    function Get-PassDirs {
        if (Test-Path $store) {
            Get-ChildItem -Path $store -Directory -Recurse -ErrorAction SilentlyContinue |
                ForEach-Object {
                    $_.FullName.Substring($store.Length + 1) -replace '\\','/'
                }
        }
    }

    function Result($v, $desc) {
        [System.Management.Automation.CompletionResult]::new($v, $v, 'ParameterValue', ($desc ?? $v))
    }

    $allCmds = 'init','ls','list','show','insert','add','generate','rm','delete','remove',
               'find','search','grep','edit','cp','copy','mv','rename','git','completion','help'

    if ($elements.Count -le 1) {
        # Subcommands + password names — `pass <tab>` also acts as show
        $allCmds | Where-Object { $_ -like "$wordToComplete*" } |
            ForEach-Object { Result $_ $_ }
        Get-PassNames | Where-Object { $_ -like "$wordToComplete*" } |
            ForEach-Object { Result $_ "Show password" }
        return
    }

    $cmd = $elements[1].Value

    switch -Wildcard ($cmd) {
        { $_ -in 'show','insert','add','generate','rm','delete','remove','edit','cp','copy','mv','rename','find','search' } {
            if ($wordToComplete -like '-*') {
                $flags = switch ($cmd) {
                    'show'                            { '--clip','--line' }
                    { $_ -in 'insert','add' }         { '--echo','--multiline','--force' }
                    'generate'                        { '--no-symbols','--clip','--in-place','--force' }
                    { $_ -in 'rm','delete','remove' } { '--recursive','--force' }
                    { $_ -in 'cp','copy','mv','rename' } { '--force' }
                    default                           { @() }
                }
                $flags | Where-Object { $_ -like "$wordToComplete*" } |
                    ForEach-Object { Result $_ $_ }
            } else {
                Get-PassNames | Where-Object { $_ -like "$wordToComplete*" } |
                    ForEach-Object { Result $_ $_ }
            }
        }
        { $_ -in 'ls','list' } {
            Get-PassDirs | Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object { Result $_ $_ }
        }
        'completion' {
            'bash','zsh','fish','powershell','elvish' |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object { Result $_ $_ }
        }
        default {
            # Unknown first token — treat as password name (mirrors the binary's default-show)
            Get-PassNames | Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object { Result $_ "Show password" }
        }
    }
}
"#;

// ---------------------------------------------------------------------------
// Elvish
// ---------------------------------------------------------------------------
const ELVISH: &str = r#"
set edit:completion:arg-completer[pass] = {|@args|
    var store = (if (has-env PASSWORD_STORE_DIR) { get-env PASSWORD_STORE_DIR } else { put ~/.password-store })

    fn pass-names {
        find $store -name "*.gpg" 2>/dev/null | each {|f|
            str:trim-suffix (str:trim-prefix $f $store"/") ".gpg"
        }
    }

    fn pass-dirs {
        find $store -mindepth 1 -type d 2>/dev/null | each {|d|
            str:trim-prefix $d $store"/"
        }
    }

    var n = (count $args)
    if (== $n 2) {
        # Subcommands + password names at position 1
        put init ls list show insert add generate rm delete remove find search grep edit cp copy mv rename git completion help
        pass-names
    } elif (>= $n 3) {
        var cmd = $args[1]
        if (or (eq $cmd show) (eq $cmd insert) (eq $cmd add) \
               (eq $cmd generate) (eq $cmd rm) (eq $cmd delete) (eq $cmd remove) \
               (eq $cmd edit) (eq $cmd cp) (eq $cmd copy) (eq $cmd mv) (eq $cmd rename) \
               (eq $cmd find) (eq $cmd search)) {
            pass-names
        } elif (or (eq $cmd ls) (eq $cmd list)) {
            pass-dirs
        } elif (eq $cmd completion) {
            put bash zsh fish powershell elvish
        }
    }
}
"#;
