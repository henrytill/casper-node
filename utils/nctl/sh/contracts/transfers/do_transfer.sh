#!/usr/bin/env bash
#
# Dispatches wasmless transfers to a test net.
# Globals:
#   NCTL - path to nctl home directory.
# Arguments:
#   Network ordinal identifier (optional).
#   Node ordinal identifier (optional).
#   User ordinal identifier (optional).
#   Transfer amount (optional).
#   Transfer dispatch count (optional).
#   Transfer dispatch interval (optional).
#   Gas price (optional).
#   Gas payment (optional).

#######################################
# Destructure input args.
#######################################

# Unset to avoid parameter collisions.
unset AMOUNT
unset GAS
unset TRANSFER_INTERVAL
unset NET_ID
unset NODE_ID
unset PAYMENT
unset TRANSFERS
unset USER_ID

for ARGUMENT in "$@"
do
    KEY=$(echo $ARGUMENT | cut -f1 -d=)
    VALUE=$(echo $ARGUMENT | cut -f2 -d=)
    case "$KEY" in
        amount) AMOUNT=${VALUE} ;;
        gas) GAS=${VALUE} ;;
        interval) TRANSFER_INTERVAL=${VALUE} ;;
        net) NET_ID=${VALUE} ;;
        node) NODE_ID=${VALUE} ;;
        payment) PAYMENT=${VALUE} ;;
        transfers) TRANSFERS=${VALUE} ;;
        user) USER_ID=${VALUE} ;;
        *)
    esac
done

# Set defaults.
AMOUNT=${AMOUNT:-$NCTL_DEFAULT_TRANSFER_AMOUNT}
PAYMENT=${PAYMENT:-$NCTL_DEFAULT_GAS_PAYMENT}
GAS=${GAS:-$NCTL_DEFAULT_GAS_PRICE}
NET_ID=${NET_ID:-1}
NODE_ID=${NODE_ID:-1}
TRANSFERS=${TRANSFERS:-100}
TRANSFER_INTERVAL=${TRANSFER_INTERVAL:-0.01}
USER_ID=${USER_ID:-1}

#######################################
# Main
#######################################

# Import utils.
source $NCTL/sh/utils.sh

# Import net vars.
source $(get_path_to_net_vars $NET_ID)

# Set deploy params.
CHAIN_NAME=$(get_chain_name $NET_ID)
CP1_SECRET_KEY=$(get_path_to_secret_key $NET_ID $NCTL_ACCOUNT_TYPE_FAUCET)
CP1_PUBLIC_KEY=$(get_account_key $NET_ID $NCTL_ACCOUNT_TYPE_FAUCET)
CP2_PUBLIC_KEY=$(get_account_key $NET_ID $NCTL_ACCOUNT_TYPE_USER $USER_ID)
PATH_TO_CLIENT=$(get_path_to_client $NET_ID)

# Inform.
log "dispatching $TRANSFERS wasmless transfers"
log "... network=$NET_ID"
log "... node=$NODE_ID"
log "... transfer amount=$AMOUNT"
log "... transfer interval=$TRANSFER_INTERVAL (s)"
log "... counter-party 1 public key=$CP1_PUBLIC_KEY"
log "... counter-party 2 public key=$CP2_PUBLIC_KEY"
log "... dispatched deploys:"

# Deploy dispatcher.
function _dispatch_deploy() {
    echo $(
        $PATH_TO_CLIENT transfer \
            --chain-name $CHAIN_NAME \
            --gas-price $GAS \
            --node-address $NODE_ADDRESS \
            --payment-amount $PAYMENT \
            --secret-key $CP1_SECRET_KEY \
            --ttl "1day" \
            --amount $AMOUNT \
            --target-account $CP2_PUBLIC_KEY \
            | jq '.result.deploy_hash' \
            | sed -e 's/^"//' -e 's/"$//'
        )
}

# Dispatch transfers to each node in round-robin fashion.
if [ $NODE_ID = "all" ]; then
    TRANSFERRED=0
    while [ $TRANSFERRED -lt $TRANSFERS ];
    do
        for IDX in $(seq 1 $NCTL_NET_NODE_COUNT)
        do
            NODE_ADDRESS=$(get_node_address_rpc $NET_ID $IDX)
            DEPLOY_HASH=$(_dispatch_deploy)
            TRANSFERRED=$((TRANSFERRED + 1))
            log "... ... #$TRANSFERRED :: $DEPLOY_HASH"
            if [[ $TRANSFERRED -eq $TRANSFERS ]]; then
                break
            fi
            sleep $TRANSFER_INTERVAL
        done
    done

# Dispatch transfers to a specific node.
else
    NODE_ADDRESS=$(get_node_address_rpc $NET_ID $NODE_ID)
    for IDX in $(seq 1 $TRANSFERS)
    do
        DEPLOY_HASH=$(_dispatch_deploy)
        log "... ... #$IDX :: $DEPLOY_HASH"
        sleep $TRANSFER_INTERVAL
    done
fi

log "dispatched $TRANSFERS wasmless transfers"