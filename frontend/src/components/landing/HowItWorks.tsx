const steps = [
  {
    num: "01",
    title: "Build a Cascade List",
    body: (
      <>
        Add up to 200 GitHub repos, Stellar addresses, or other Cascade Lists, each with a
        percentage weight. Lists are just Soroban contract state — anyone can view the split
        before funding it.
      </>
    ),
  },
  {
    num: "02",
    title: "Fund it your way",
    body: (
      <>
        Choose a{" "}
        <code className="rounded-sm bg-[#14261E] px-1.5 py-0.5 font-['IBM_Plex_Mono'] text-[13px] text-[#34D399]">
          Stream
        </code>{" "}
        for continuous, adjustable funding, or a{" "}
        <code className="rounded-sm bg-[#14261E] px-1.5 py-0.5 font-['IBM_Plex_Mono'] text-[13px] text-[#34D399]">
          Give
        </code>{" "}
        for a one-time transfer. Both settle on Soroban with roughly five-second finality.
      </>
    ),
  },
  {
    num: "03",
    title: "Maintainers claim & cascade",
    body: (
      <>
        A maintainer adds a{" "}
        <code className="rounded-sm bg-[#14261E] px-1.5 py-0.5 font-['IBM_Plex_Mono'] text-[13px] text-[#34D399]">
          cascade.json
        </code>{" "}
        to their default branch, claims their share, and optionally forwards a cut to their own
        dependencies — which repeats down the tree.
      </>
    ),
  },
];

export default function HowItWorks() {
  return (
    <section id="how" className="border-y border-[#1E2E27] bg-[#0F1D17] px-8 py-26">
      <div className="mx-auto max-w-[1180px]">
        <span className="mb-4 block font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.1em] text-[#34D399]">
          How it works
        </span>
        <h2 className="max-w-[660px] font-['IBM_Plex_Mono'] text-[28px] font-semibold leading-tight tracking-[-0.01em] text-[#EAF3EE] sm:text-[42px]">
          Three permissionless steps, no one blocking the next.
        </h2>
        <p className="mt-4.5 max-w-[560px] text-base leading-relaxed text-[#7C9188]">
          Every step can be triggered by anyone — a funder, a maintainer, or a bot watching the
          chain. Nothing waits on a single party to act.
        </p>

        <div className="mt-16 grid grid-cols-1 gap-px border border-[#1E2E27] bg-[#1E2E27] md:grid-cols-3">
          {steps.map((step) => (
            <div key={step.num} className="bg-[#0F1D17] px-8 py-10">
              <span className="mb-5 block font-['IBM_Plex_Mono'] text-[13px] text-[#F5B942]">
                {step.num}
              </span>
              <h3 className="mb-3 text-[19px] tracking-[-0.01em] text-[#EAF3EE]">{step.title}</h3>
              <p className="text-[14.5px] leading-relaxed text-[#7C9188]">{step.body}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
