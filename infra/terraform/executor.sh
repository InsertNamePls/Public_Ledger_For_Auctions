#!/bin/bash

main() {
	rm -rf target
	mkdir target
	cp env.tfvars target/env.tfvars
	cp common/* target/
	cp "$module"/* target/
	cd target
	init
	if [ "$execution_type" == "plan" ]; then
		plan
	elif [ "$execution_type" == "apply" ]; then
		apply
	elif [ "$execution_type" == "destroy" ]; then
		destroy
	else
		echo "exec type => $execution_type not valid"
	fi
}

init() {
	terraform init \
		-var-file=env.tfvars \
		-backend-config='prefix=terraform/state/'$module &&
		terraform fmt --check && \ 
	terraform validate

}
plan() {
	terraform plan \
		-var-file=env.tfvars
}

apply() {
	terraform apply \
		-var-file=env.tfvars #-auto-approve
}
destroy() {
	terraform destroy \
		-var-file=env.tfvars -auto-approve
}

if [[ ! -n $1 ]]; then
	echo "You must especify a module to be applied"
	exit 0
else
	module=$1
	execution_type=$2
	main
	echo $module $execution_type
fi
