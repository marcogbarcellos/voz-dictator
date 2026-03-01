export const APP_NAME = "Voz";
export const APP_VERSION = "0.1.0";

export const PRICING = {
  free: { name: "Free", price: 0, transcriptions: 50 },
  pro: { name: "Pro", price: 8, transcriptions: Infinity },
  lifetime: { name: "Lifetime", price: 79, transcriptions: Infinity },
} as const;

export type PricingTier = keyof typeof PRICING;
