export default function Cta() {
  return (
    <section
      id="start"
      className="border-t border-[#1E2E27] bg-[#0A1410] bg-[radial-gradient(circle_at_50%_0%,rgba(52,211,153,0.12),transparent_55%)] px-8 py-26 text-center"
    >
      <div className="mx-auto max-w-[1180px]">
        <span className="mb-4 block font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.1em] text-[#34D399]">
          Get started
        </span>
        <h2 className="mx-auto mb-5.5 max-w-[720px] font-['IBM_Plex_Mono'] text-[30px] font-semibold leading-tight text-[#EAF3EE] sm:text-[52px]">
          Fund the chain, not just the repo.
        </h2>
        <p className="mx-auto mb-10 max-w-[480px] text-base text-[#7C9188]">
          Cascade is live on Stellar Testnet. Claim your repository or start a Cascade List for
          the projects you depend on.
        </p>
        <div className="flex flex-wrap justify-center gap-4">
          <a
            href="#"
            className="inline-flex items-center gap-2.5 rounded-[3px] bg-[#34D399] px-6.5 py-4 font-['IBM_Plex_Mono'] text-sm font-semibold text-[#0A1410] transition-all hover:-translate-y-0.5 hover:brightness-105"
          >
            Claim your repo →
          </a>
          <a
            href="#"
            className="inline-flex items-center gap-2.5 rounded-[3px] border border-[#1E2E27] px-6.5 py-4 font-['IBM_Plex_Mono'] text-sm font-medium text-[#EAF3EE] transition-colors hover:border-[#34D399] hover:bg-[#34D399]/10"
          >
            Read the docs
          </a>
        </div>
      </div>
    </section>
  );
}
