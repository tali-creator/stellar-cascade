import CascadeDiagram from "./CascadeDiagram";

export default function Hero() {
  return (
    <section
      id="top"
      className="relative bg-[#0A1410] bg-[radial-gradient(circle_at_82%_12%,rgba(52,211,153,0.12),transparent_45%)] px-8 pb-24 pt-46"
    >
      <div className="mx-auto max-w-[1180px]">
        <div className="mb-6 inline-flex items-center gap-2.5 rounded-full border border-[#1E2E27] px-3 py-1.5 font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.08em] text-[#F5B942]">
          <span className="h-1.5 w-1.5 rounded-full bg-[#F5B942]" />
          Live on Stellar Testnet · Soroban smart contracts
        </div>

        <h1 className="mb-7 max-w-[820px] font-['IBM_Plex_Mono'] text-[38px] font-semibold leading-[1.06] tracking-[-0.02em] text-[#EAF3EE] sm:text-[54px] lg:text-[74px]">
          Funding that <span className="text-[#34D399]">cascades</span> through your dependency tree.
        </h1>

        <p className="mb-11 max-w-[600px] text-lg leading-relaxed text-[#7C9188]">
          Cascade lets anyone stream or gift funds to a list of GitHub repos and wallets. When a
          maintainer claims their share, they declare their own dependencies — and a cut flows
          further downstream automatically, with no one holding up the chain.
        </p>

        <div className="mb-5 flex flex-wrap gap-4">
          <a
            href="#start"
            className="inline-flex items-center gap-2.5 rounded-[3px] bg-[#34D399] px-6.5 py-4 font-['IBM_Plex_Mono'] text-sm font-semibold text-[#0A1410] transition-all hover:-translate-y-0.5 hover:brightness-105"
          >
            Create a Cascade List →
          </a>
          <a
            href="#how"
            className="inline-flex items-center gap-2.5 rounded-[3px] border border-[#1E2E27] px-6.5 py-4 font-['IBM_Plex_Mono'] text-sm font-medium text-[#EAF3EE] transition-colors hover:border-[#34D399] hover:bg-[#34D399]/10"
          >
            See how splitting works
          </a>
        </div>

        <p className="font-['IBM_Plex_Mono'] text-xs text-[#7C9188]">
          No wallet setup required to claim — sign in with GitHub, verify with a cascade.json file.
        </p>

        <CascadeDiagram />
      </div>
    </section>
  );
}
