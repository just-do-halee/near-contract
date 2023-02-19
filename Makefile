accountId = 
contractId = 
# ------------------------------------------------------
# 
# set : Sets the account and contract id
#
# new : Creates a new contract id
#
# del : Deletes the contract id
#
# state : Gets the state of the contract
#
# send : Sends tokens to the contract [amount=]
#
# test : Tests the contract code
#
# build : Builds the contract code ( + test )
#
# deploy : Deploys the contract code ( + build )
#
# all : deploy 
#
# call : Calls the contract method [method=,args=]
#
# view : Calls the contract view method [method=,args=]
#
# clean : Removes the contract *.wasm files with 'cargo clean'
#
# ------------------------------------------------------
MKPATH = $(abspath $(lastword $(MAKEFILE_LIST)))
BUILDPATH = build

C=cargo
R=rustup
CFLAG=RUSTFLAG
CTARGET=wasm32-unknown-unknown
N=near

all: deploy

checkAccount:
ifeq ($(and $(accountId),$(contractId)),)
	$(error make set accountId=.. contractId=.. Or make new newContractId=..)
else
	@echo "accountId: ${accountId}\ncontractId: ${contractId}\n"
endif

new:
ifdef accountId
else
	$(error accountId is not set)
endif
ifdef newContractId
else
	$(error newContractId is not set)
endif
	$(N) create-account $(newContractId) --masterAccount=$(accountId) \
		$(if $(initialBalance), --initialBalance=$(initialBalance)) \
		$(if $(contractName), --contractName=$(contractName)) \
		$(if $(helperUrl), --helperUrl=$(helperUrl)) \
		$(if $(networkId), --networkId=$(networkId)) \
		$(if $(nodeUrl), --nodeUrl=$(nodeUrl)) \
		$(if $(keyPath), --keyPath=$(keyPath)) \
		$(if $(useLedgerKey), --useLedgerKey=$(useLedgerKey)) \
		$(if $(publicKey), --publicKey=$(publicKey)) \
		$(if $(seedPhrase), --seedPhrase=$(seedPhrase)) \
		$(if $(seedPath), --seedPath=$(seedPath)) \
		$(if $(walletUrl), --walletUrl=$(walletUrl)) \
		$(if $(helperAccount), --helperAccount=$(helperAccount)) \
		$(if $(verbose), --verbose=$(verbose)) \
		$(if $(force), --force=$(force)) \
		$(if $(help), --help=$(help)) \
		$(if $(version), --version=$(version))
	@{ echo "accountId = ${accountId}"; echo "contractId = ${newContractId}"; tail -n +3 $(MKPATH); } > $(MKPATH).tmp && mv $(MKPATH).tmp $(MKPATH)

set: checkAccount
	@{ echo "accountId = ${accountId}"; echo "contractId = ${contractId}"; tail -n +3 $(MKPATH); } > $(MKPATH).tmp && mv $(MKPATH).tmp $(MKPATH) 

del: checkAccount
	@echo "deleting ${contractId}\n"
	@$(N) delete $(contractId) $(accountId)

state: checkAccount
	@echo "state of ${contractId}\n"
	@$(N) state $(contractId)

send: checkAccount
ifdef amount
else
	$(error amount is not set)
endif
	@echo "sending ${amount} to ${contractId}\n"
	@$(N) send $(accountId) $(contractId) $(amount)


# ------------------------------------------------------

test:
	@$(C) test -- --nocapture

build: test
	@$(R) target add $(CTARGET)
	@$(CFLAG)='-C link-arg=-s' $(C) build --release --target=$(CTARGET)
	@if [ ! -d "$(BUILDPATH)" ]; then mkdir $(BUILDPATH); fi
	@cp ./target/$(CTARGET)/release/*.wasm ./$(BUILDPATH)/

deploy: checkAccount test build
	@echo "deploying ${contractId} with ${accountId} \ninitFunction = ${initFunction}\ninitArgs = ${initArgs}\ninitGas = ${initGas}\ninitDeposit = ${initDeposit}\n"
	@$(N) deploy $(contractId) --wasmFile ./$(BUILDPATH)/*.wasm \
		$(if $(initFunction), --initFunction=$(initFunction)) \
		$(if $(initArgs), --initArgs='$(initArgs)') \
		$(if $(initGas), --initGas=$(initGas)) \
		$(if $(initDeposit), --initDeposit=$(initDeposit))

# ------------------------------------------------------

checkMethod:
ifdef method
else
	$(error method is not set)
endif

call: checkAccount checkMethod
	@$(N) call $(contractId) $(method) '$(args)' --accountId=$(accountId)

view: checkAccount checkMethod
	@$(N) view $(contractId) $(method) '$(args)'

# ------------------------------------------------------

clean:
	@rm -rf ./$(BUILDPATH)/*
	@$(C) clean


# ------------------------------------------------------
#
# LICENSE
#
# "MIT OR Apache-2.0"
#
# by Doha Lee(= Hwakyeom Kim) <just.do.halee@gmail.com>
