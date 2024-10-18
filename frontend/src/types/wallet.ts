export type LnPayState =
  | "created"
  | "canceled"
  | { funded: { block_height: number } }
  | { waiting_for_refund: { error_reason: string } }
  | "awaiting_change"
  | { Success: { preimage: string } }
  | { refunded: { gateway_error: string } }
  | { unexpected_error: { error_message: string } };

export type LnReceiveState =
  | "created"
  | { waiting_for_payment: { invoice: string; timeout: number } }
  | { canceled: { reason: string } }
  | "funded"
  | "awaiting_funds"
  | "claimed";
