import { FedimintWallet } from "@fedimint/core-web";

const FEDERATION_INVITE_CODE =
  "fed11qgqrgvnhwden5te0v9k8q6rp9ekh2arfdeukuet595cr2ttpd3jhq6rzve6zuer9wchxvetyd938gcewvdhk6tcqqysptkuvknc7erjgf4em3zfh90kffqf9srujn6q53d6r056e4apze5cw27h75";

const wallet = new FedimintWallet();

wallet.setLogLevel("info");

const isOpen = await wallet.open();

console.log("Wallet is open:", isOpen);

if (!isOpen) {
  await wallet.joinFederation(FEDERATION_INVITE_CODE);
}

await wallet.waitForOpen();

export { wallet };
