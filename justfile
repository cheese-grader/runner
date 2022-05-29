default:
    just --list

build-all:
    #!/bin/sh
    for lang in ./docker/* ; do
        just build $(basename $lang)

        if [ $? -ne 0 ]
            then break
        fi
    done

build LANGS:
    #!/bin/sh
    IFS="," read -ra langs <<< "{{ LANGS }}"
    for lang in ${langs[@]} ; do
        echo "Building $lang..."
        docker build -t cheese-grader/runner-$lang ./docker/$lang

        if [ $? -ne 0 ]
            then break
        fi
    done

    echo "Built ${#langs[@]} languages"

test-all:
    #!/bin/sh
    for lang in ./docker/* ; do
        just build $(basename $lang)
        just test $(basename $lang)

        if [ $? -ne 0 ]
            then break
        fi
    done
