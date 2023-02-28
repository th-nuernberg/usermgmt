#!/bin/bash
# no strict mode like set -eu
# Reason: sacctmgr returns non zero if a spec to be added, like account student, is already there

prefix='sacctmgr --immediate'


$prefix add account thn Description="THN root account" Organization=thn 
$prefix add account cs parent=thn Description="Computer Science faculty accounts" Organization=thn
$prefix add account staff parent=cs Description="Computer Science faculty staff accounts" Organization=thn
$prefix add account student parent=cs Description="Computer Science faculty student accounts" Organization=thn

# Add QOSs
$prefix add qos interactive
$prefix add qos basic
$prefix add qos advanced
$prefix add qos ultimate
$prefix add qos bigmem
$prefix add qos gpubasic
$prefix add qos gpuultimate
$prefix modify qos interactive set MaxJobsPerUser=2 MaxWallDurationPerJob=360:00 MaxTRESPerUser=cpu=4,mem=16384,gres/gpu=1
$prefix modify qos basic set MaxJobsPerUser=10 MaxWallDurationPerJob=720:00 MaxTRESPerUser=cpu=16,mem=32768,gres/gpu=1
$prefix modify qos advanced set MaxJobsPerUser=20 MaxWallDurationPerJob=1440:00 MaxTRESPerUser=cpu=24,mem=49152,gres/gpu=1
$prefix modify qos ultimate set MaxJobsPerUser=50 MaxWallDurationPerJob=2880:00 MaxTRESPerUser=cpu=32,mem=65536,gres/gpu=1
$prefix modify qos bigmem set MaxJobsPerUser=10 MaxWallDurationPerJob=720:00 MaxTRESPerUser=cpu=16,mem=131072
$prefix modify qos gpubasic set MaxJobsPerUser=10 MaxWallDurationPerJob=5760:00 MaxTRESPerUser=cpu=8,mem=16384,gres/gpu=1
$prefix modify qos gpuultimate set MaxJobsPerUser=10 MaxWallDurationPerJob=2880:00 MaxTRESPerUser=cpu=16,mem=32768,gres/gpu=3

