interface BalanceDisplayProps {
  balance: number;
}

export default function BalanceDisplay({ balance }: BalanceDisplayProps) {
  return (
    <div className="text-center my-4">
      <h2 className="text-2xl font-bold">Balance</h2>
      <p className="text-4xl font-bold text-primary">{balance} sats</p>
    </div>
  );
}
