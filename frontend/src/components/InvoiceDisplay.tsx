import QRCode from "react-qr-code";
import { CheckCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface InvoiceDisplayProps {
  invoice: string;
  paid: boolean;
}

export function InvoiceDisplay({ invoice, paid }: InvoiceDisplayProps) {
  return (
    <div className="flex flex-col items-center space-y-6">
      <div className="bg-white p-4 rounded-lg shadow-md">
        <QRCode value={invoice} size={256} />
      </div>
      {paid ? (
        <div className="text-green-500 animate-bounce">
          <CheckCircle size={48} />
        </div>
      ) : (
        <div className="w-full space-y-2">
          <Label htmlFor="invoice">Lightning Invoice</Label>
          <div className="flex">
            <Input
              id="invoice"
              value={invoice}
              readOnly
              className="flex-grow"
            />
            <Button
              onClick={() => navigator.clipboard.writeText(invoice)}
              className="ml-2"
            >
              Copy
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
