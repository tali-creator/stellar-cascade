const rows = [
  {
    label: "Settlement frequency",
    value: "Near-continuous — no need to batch monthly to control fees",
    accent: true,
  },
  { label: "Finality", value: "~5 seconds via Stellar Consensus Protocol", accent: false },
  {
    label: "Cost per split trigger",
    value: "Fractions of a cent, even for deep dependency chains",
    accent: true,
  },
  {
    label: "Claiming identity",
    value: "GitHub-verified cascade.json, no prior wallet needed",
    accent: false,
  },
  {
    label: "List size limit",
    value: "200 by default — a deliberate, adjustable choice, not a gas-driven ceiling",
    accent: false,
  },
];

export default function WhySoroban() {
  return (
    <section id="compare" className="border-t border-[#1E2E27] bg-[#0F1D17] px-8 py-26">
      <div className="mx-auto max-w-[1180px]">
        <span className="mb-4 block font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.1em] text-[#34D399]">
          Why Soroban
        </span>
        <h2 className="max-w-[660px] font-['IBM_Plex_Mono'] text-[28px] font-semibold leading-tight tracking-[-0.01em] text-[#EAF3EE] sm:text-[42px]">
          The constraints that shape batching mostly disappear.
        </h2>
        <p className="mt-4.5 max-w-[560px] text-base leading-relaxed text-[#7C9188]">
          Splitting protocols on other chains batch settlement to control gas costs. Soroban&apos;s
          fee and finality profile removes most of that pressure.
        </p>

        <div className="mt-14 overflow-hidden rounded-lg border border-[#1E2E27]">
          <div className="grid grid-cols-1 border-b border-[#1E2E27] bg-[#14261E] md:grid-cols-2">
            <div className="px-7 py-5 font-['IBM_Plex_Mono'] text-[13px] font-semibold uppercase tracking-[0.05em] text-[#EAF3EE]">
              Constraint
            </div>
            <div className="px-7 py-5 font-['IBM_Plex_Mono'] text-[13px] font-semibold uppercase tracking-[0.05em] text-[#EAF3EE]">
              On Soroban
            </div>
          </div>

          {rows.map((row, i) => (
            <div
              key={row.label}
              className={`grid grid-cols-1 md:grid-cols-2 ${
                i !== rows.length - 1 ? "border-b border-[#1E2E27]" : ""
              }`}
            >
              <div className="border-b border-[#1E2E27] px-7 py-5 font-['IBM_Plex_Mono'] text-[12.5px] uppercase tracking-[0.05em] text-[#7C9188] md:border-b-0">
                {row.label}
              </div>
              <div
                className={`px-7 py-5 text-sm leading-relaxed ${
                  row.accent ? "text-[#34D399]" : "text-[#EAF3EE]"
                }`}
              >
                {row.value}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
