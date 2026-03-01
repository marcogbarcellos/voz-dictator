import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { getUsageSummary, type UsageSummary } from "../lib/tauri-commands";

function formatCost(cost: number): string {
  if (cost < 0.01) return `$${cost.toFixed(4)}`;
  if (cost < 1) return `$${cost.toFixed(3)}`;
  return `$${cost.toFixed(2)}`;
}

export function UsageBar() {
  const [summary, setSummary] = useState<UsageSummary | null>(null);
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    let mounted = true;

    async function fetch() {
      try {
        const data = await getUsageSummary();
        if (mounted) setSummary(data);
      } catch {
        // ignore
      }
    }

    fetch();
    const interval = setInterval(fetch, 30_000);
    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, []);

  if (!summary || summary.total_calls === 0) return null;

  return (
    <div className="px-5 py-2 border-t border-glass-border">
      <button
        onClick={() => setExpanded((v) => !v)}
        className="w-full flex items-center justify-between text-[11px] text-text-muted hover:text-text-secondary transition-colors"
      >
        <div className="flex items-center gap-2">
          <svg className="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
            <path d="M12 2v20M17 5H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H6" />
          </svg>
          <span>
            {formatCost(summary.today_cost)} today
            <span className="text-text-muted/60 mx-1">|</span>
            {formatCost(summary.total_cost)} total
          </span>
        </div>
        <motion.svg
          animate={{ rotate: expanded ? 180 : 0 }}
          transition={{ duration: 0.15 }}
          className="w-3 h-3"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth={2}
        >
          <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
        </motion.svg>
      </button>

      <AnimatePresence>
        {expanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.15 }}
            className="overflow-hidden"
          >
            <div className="pt-2 space-y-1">
              {summary.by_provider.map((p) => (
                <div
                  key={p.provider}
                  className="flex items-center justify-between text-[11px]"
                >
                  <span className="text-text-secondary capitalize">{p.provider}</span>
                  <span className="text-text-muted">
                    {formatCost(p.cost)}
                    <span className="text-text-muted/50 ml-1">({p.calls})</span>
                  </span>
                </div>
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
