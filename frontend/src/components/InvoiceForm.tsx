import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface InvoiceFormProps {
  onSubmit: (amount: number) => Promise<void>;
  onCancel: () => void;
  generating: boolean;
}

export function InvoiceForm({
  onSubmit,
  onCancel,
  generating,
}: InvoiceFormProps) {
  const [amount, setAmount] = useState("");

  useEffect(() => {
    console.log("Component mounted or updated");
    console.log("Current amount:", amount);
    console.log("Generating state:", generating);
  }, [amount, generating]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    console.log("Form submitted");
    const numAmount = parseFloat(amount);
    console.log("Parsed amount:", numAmount);
    if (!isNaN(numAmount) && numAmount > 0) {
      console.log("Calling onSubmit with amount:", numAmount);
      onSubmit(numAmount);
    } else {
      console.log("Invalid input detected");
      alert("Please enter a valid amount greater than 0");
    }
  };

  const handleAmountChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newAmount = e.target.value;
    console.log("Amount changed to:", newAmount);
    setAmount(newAmount);
  };

  console.log(
    "Rendering component, amount:",
    amount,
    "generating:",
    generating
  );

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <Label htmlFor="amount">Amount (sats)</Label>
        <Input
          id="amount"
          type="number"
          placeholder="Enter amount to receive"
          value={amount}
          onChange={handleAmountChange}
          className="mt-1"
          required
          min="1"
        />
      </div>
      <div className="flex justify-between gap-4">
        <Button
          type="submit"
          className="flex-1"
          disabled={generating || !amount}
          onClick={() => console.log("Generate Invoice button clicked")}
        >
          {generating ? "Generating..." : "Generate Invoice"}
        </Button>
        <Button
          type="button"
          onClick={() => {
            console.log("Cancel button clicked");
            onCancel();
          }}
          className="flex-1"
          variant="outline"
        >
          Cancel
        </Button>
      </div>
    </form>
  );
}
