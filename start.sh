#!/bin/sh

set -ex
./sqlx database setup
./drop
