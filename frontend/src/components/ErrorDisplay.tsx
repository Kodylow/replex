interface ErrorDisplayProps {
  error: string | null;
}

export function ErrorDisplay({ error }: ErrorDisplayProps) {
  if (!error) return null;
  return <div className="text-red-500 mt-4 text-center">{error}</div>;
}
