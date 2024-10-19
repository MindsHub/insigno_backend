#!/bin/bash

backup(){
    sudo docker run -it --rm -v $1:/volume -v $(pwd):/backup --name dbstore ubuntu tar cf /backup/backup/backup-$1.tar /volume
}

restore(){
    sudo docker run -it --rm -v $1:/volume -v $(pwd):/backup --name dbstore ubuntu tar xf /backup/backup/backup-$1.tar -C /volume
}
mkdir backup

backup docker_db-vol
backup docker_grafana-vol
backup docker_media-vol
backup docker_prometheus-vol
backup docker_caddy-data-vol

#restore docker_db-vol
#restore docker_grafana-vol
#restore docker_media-vol
#restore docker_caddy-data-vol
#restore docker_caddy-data-vol