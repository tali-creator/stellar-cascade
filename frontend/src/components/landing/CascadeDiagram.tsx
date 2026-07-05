import "./animations.css";

export default function CascadeDiagram() {
  return (
    <div className="mt-22 overflow-x-auto rounded-lg border border-[#1E2E27] bg-[#0F1D17] p-10 sm:p-14">
      <span className="mb-8 block font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.08em] text-[#7C9188]">
        Funds settling through a live dependency tree
      </span>

      <svg viewBox="0 0 900 260" fill="none" className="h-auto min-w-[640px] w-full">
        {/* root */}
        <rect x="380" y="16" width="140" height="44" rx="4" className="fill-[#14261E] stroke-[#34D399]" />
        <text x="450" y="34" textAnchor="middle" fill="#EAF3EE" fontFamily="IBM Plex Mono" fontSize="12" fontWeight={600}>
          your-app
        </text>
        <text x="450" y="49" textAnchor="middle" fill="#34D399" fontFamily="IBM Plex Mono" fontSize="10">
          stream · 500 USDC/mo
        </text>

        {/* level 2 */}
        <rect x="180" y="120" width="150" height="44" rx="4" className="fill-[#14261E] stroke-[#34D399]" />
        <text x="255" y="138" textAnchor="middle" fill="#EAF3EE" fontFamily="IBM Plex Mono" fontSize="12" fontWeight={600}>
          http-router
        </text>
        <text x="255" y="153" textAnchor="middle" fill="#7C9188" fontFamily="IBM Plex Mono" fontSize="10">
          60% claimed
        </text>

        <rect x="380" y="120" width="150" height="44" rx="4" className="fill-[#14261E] stroke-[#34D399]" />
        <text x="455" y="138" textAnchor="middle" fill="#EAF3EE" fontFamily="IBM Plex Mono" fontSize="12" fontWeight={600}>
          auth-toolkit
        </text>
        <text x="455" y="153" textAnchor="middle" fill="#7C9188" fontFamily="IBM Plex Mono" fontSize="10">
          25% claimed
        </text>

        <rect x="580" y="120" width="150" height="44" rx="4" className="fill-[#14261E] stroke-[#1E2E27]" />
        <text x="655" y="138" textAnchor="middle" fill="#EAF3EE" fontFamily="IBM Plex Mono" fontSize="12" fontWeight={600}>
          test-runner
        </text>
        <text x="655" y="153" textAnchor="middle" fill="#7C9188" fontFamily="IBM Plex Mono" fontSize="10">
          15% unclaimed
        </text>

        {/* level 3 */}
        <rect x="120" y="204" width="140" height="44" rx="4" className="fill-[#14261E] stroke-[#34D399]" />
        <text x="190" y="222" textAnchor="middle" fill="#EAF3EE" fontFamily="IBM Plex Mono" fontSize="12" fontWeight={600}>
          tls-parser
        </text>
        <text x="190" y="237" textAnchor="middle" fill="#F5B942" fontFamily="IBM Plex Mono" fontSize="10">
          forwards 40%
        </text>

        <rect x="280" y="204" width="140" height="44" rx="4" className="fill-[#14261E] stroke-[#34D399]" />
        <text x="350" y="222" textAnchor="middle" fill="#EAF3EE" fontFamily="IBM Plex Mono" fontSize="12" fontWeight={600}>
          retry-logic
        </text>
        <text x="350" y="237" textAnchor="middle" fill="#F5B942" fontFamily="IBM Plex Mono" fontSize="10">
          forwards 20%
        </text>

        {/* connectors */}
        <path d="M450 60 L 255 120" stroke="#34D399" strokeWidth={1.6} className="flow-dash" />
        <path d="M450 60 L 455 120" stroke="#34D399" strokeWidth={1.6} className="flow-dash" />
        <path d="M450 60 L 655 120" stroke="#1E2E27" strokeWidth={1.6} />
        <path d="M255 164 L 190 204" stroke="#F5B942" strokeWidth={1.6} className="flow-dash" />
        <path d="M255 164 L 350 204" stroke="#F5B942" strokeWidth={1.6} className="flow-dash" />
      </svg>
    </div>
  );
}
