const modes = [
  {
    name: "Stream",
    description:
      "Configure a rate — say 500 USDC over 30 days — and it settles continuously. Pause, top up, or redirect it at any point; unclaimed balances simply keep accruing for the recipient.",
    meta: [
      { label: "Settlement", value: "Continuous, ~5s ticks" },
      { label: "Adjustable", value: "Anytime, by funder" },
      { label: "Best for", value: "Ongoing maintenance" },
    ],
  },
  {
    name: "Give",
    description:
      "Send a fixed amount into a list right now, no streaming config required. It splits and cascades through dependencies the moment someone triggers settlement.",
    meta: [
      { label: "Settlement", value: "Single transaction" },
      { label: "Adjustable", value: "N/A — instant" },
      { label: "Best for", value: "Grants, one-off bounties" },
    ],
  },
];

export default function FundingModes() {
  return (
    <section id="modes" className="bg-[#0A1410] px-8 py-26">
      <div className="mx-auto max-w-[1180px]">
        <span className="mb-4 block font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.1em] text-[#34D399]">
          Funding modes
        </span>
        <h2 className="max-w-[660px] font-['IBM_Plex_Mono'] text-[28px] font-semibold leading-tight tracking-[-0.01em] text-[#EAF3EE] sm:text-[42px]">
          Stream it continuously, or give it once.
        </h2>
        <p className="mt-4.5 max-w-[560px] text-base leading-relaxed text-[#7C9188]">
          Both modes settle through the same Cascade List and split the same way downstream — the
          only difference is how the funds arrive.
        </p>

        <div className="mt-16 grid grid-cols-1 gap-6 md:grid-cols-2">
          {modes.map((mode) => (
            <div key={mode.name} className="rounded-lg border border-[#1E2E27] bg-[#0F1D17] p-10">
              <h3 className="mb-3.5 font-['IBM_Plex_Mono'] text-xl text-[#EAF3EE]">
                {mode.name}
                <span className="text-[#34D399]">()</span>
              </h3>
              <p className="mb-6 text-[14.5px] leading-relaxed text-[#7C9188]">{mode.description}</p>
              <div className="flex flex-col gap-2.5">
                {mode.meta.map((row) => (
                  <div
                    key={row.label}
                    className="flex justify-between border-t border-[#1E2E27] pt-2.5 font-['IBM_Plex_Mono'] text-[13px] text-[#7C9188]"
                  >
                    <span>{row.label}</span>
                    <b className="font-medium text-[#EAF3EE]">{row.value}</b>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
