import { useContext, useEffect, useState } from "react";
import { ReceiveContext } from "../contexts/ReceiveContext";
import { InvoiceForm } from "@/components/InvoiceForm";
import { InvoiceDisplay } from "@/components/InvoiceDisplay";
import { ErrorDisplay } from "@/components/ErrorDisplay";

interface ReceiveScreenProps {
  onComplete: () => void;
}

export default function ReceiveScreen({ onComplete }: ReceiveScreenProps) {
  const { invoice, paid, error, createInvoice, resetState } =
    useContext(ReceiveContext);
  const [generating, setGenerating] = useState(false);

  useEffect(() => {
    let timeout: NodeJS.Timeout;
    if (paid) {
      timeout = setTimeout(onComplete, 2000);
    }
    return () => clearTimeout(timeout);
  }, [paid, onComplete]);

  const handleSubmit = async (amount: number) => {
    setGenerating(true);
    try {
      await createInvoice(amount);
    } catch (error) {
      console.error("Error in createInvoice:", error);
    } finally {
      setGenerating(false);
    }
  };

  const handleCancel = () => {
    resetState();
    onComplete();
  };

  return (
    <div className="mt-4 max-w-md mx-auto">
      <h2 className="text-2xl font-bold mb-6 text-center">Receive Payment</h2>
      {!invoice ? (
        <InvoiceForm
          onSubmit={handleSubmit}
          onCancel={handleCancel}
          generating={generating}
        />
      ) : (
        <InvoiceDisplay invoice={invoice} paid={paid} />
      )}
      <ErrorDisplay error={error} />
    </div>
  );
}
