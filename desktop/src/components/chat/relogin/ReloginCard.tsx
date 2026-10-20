import { memo } from 'react';

export type ReloginReason = 'unbound' | 'bound_other';

export interface ReloginCardProps {
  reason: ReloginReason;
  onDone?: () => void;
}

export const ReloginCard = memo(function ReloginCard(_props: ReloginCardProps) {
  return null;
});
