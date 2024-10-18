import React, { useContext, useState } from "react";
import {
  WalletContext,
  SendTransaction,
  ReceiveTransaction,
} from "@/contexts/WalletContext";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ArrowUpIcon, ArrowDownIcon } from "@heroicons/react/24/solid";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

const truncateInvoice = (invoice: string) => {
  return invoice.length > 10
    ? `${invoice.slice(0, 6)}...${invoice.slice(-6)}`
    : invoice;
};

const TransactionModal: React.FC<{
  transaction: SendTransaction | ReceiveTransaction;
  isOpen: boolean;
  onClose: () => void;
}> = ({ transaction, isOpen, onClose }) => {
  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {transaction.type === "send" ? "Sent" : "Received"} Transaction
          </DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <div>
            <p className="font-semibold">Amount:</p>
            <p
              className={
                transaction.type === "send" ? "text-red-500" : "text-green-500"
              }
            >
              {transaction.type === "send" ? "-" : "+"}
              {transaction.amount} sats
            </p>
          </div>
          <div>
            <p className="font-semibold">Date:</p>
            <p>{new Date(transaction.timestamp).toLocaleString()}</p>
          </div>
          <div>
            <p className="font-semibold">Invoice:</p>
            <div className="flex items-center space-x-2">
              <p className="text-sm break-all">{transaction.invoice}</p>
              <Button
                variant="outline"
                size="sm"
                onClick={() => copyToClipboard(transaction.invoice)}
              >
                Copy
              </Button>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

const TransactionItem: React.FC<{
  transaction: SendTransaction | ReceiveTransaction;
}> = ({ transaction }) => {
  const [isModalOpen, setIsModalOpen] = useState(false);

  const openModal = () => setIsModalOpen(true);
  const closeModal = () => setIsModalOpen(false);

  return (
    <>
      <Card className="mb-2 p-2 cursor-pointer" onClick={openModal}>
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            {transaction.type === "send" ? (
              <ArrowUpIcon className="h-5 w-5 text-red-500 flex-shrink-0" />
            ) : (
              <ArrowDownIcon className="h-5 w-5 text-green-500 flex-shrink-0" />
            )}
            <p className="text-sm truncate flex-grow">
              {truncateInvoice(transaction.invoice)}
            </p>
          </div>
          <p
            className={`font-bold text-sm whitespace-nowrap ${
              transaction.type === "send" ? "text-red-500" : "text-green-500"
            }`}
          >
            {transaction.type === "send" ? "-" : "+"}
            {transaction.amount} sats
          </p>
        </div>
      </Card>
      <TransactionModal
        transaction={transaction}
        isOpen={isModalOpen}
        onClose={closeModal}
      />
    </>
  );
};

export const Transactions: React.FC = () => {
  const { state } = useContext(WalletContext);

  return (
    <div className="flex-1 bg-background">
      <ScrollArea className="h-[calc(100vh-12rem)]">
        {state.transactionHistory.length > 0 ? (
          state.transactionHistory.map((transaction) => (
            <TransactionItem key={transaction.id} transaction={transaction} />
          ))
        ) : (
          <p className="text-center mt-6 text-gray-500">No transactions yet</p>
        )}
      </ScrollArea>
    </div>
  );
};
