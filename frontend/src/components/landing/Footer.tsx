const columns = [
  {
    title: "Protocol",
    links: [
      { label: "How it works", href: "#how" },
      { label: "Streams & gives", href: "#modes" },
      { label: "Why Soroban", href: "#compare" },
    ],
  },
  {
    title: "Build",
    links: [
      { label: "GitHub", href: "#" },
      { label: "Documentation", href: "#" },
      { label: "cascade.json spec", href: "#" },
    ],
  },
  {
    title: "Community",
    links: [
      { label: "Discord", href: "#" },
      { label: "Governance", href: "#" },
      { label: "Grants & bounties", href: "#" },
    ],
  },
];

export default function Footer() {
  return (
    <footer className="border-t border-[#1E2E27] px-8 py-14">
      <div className="mx-auto max-w-[1180px]">
        <div className="mb-12 grid grid-cols-1 gap-10 sm:grid-cols-2 md:grid-cols-[1.4fr_1fr_1fr_1fr]">
          <div>
            <a
              href="#top"
              className="flex items-center gap-2.5 font-['IBM_Plex_Mono'] text-lg font-semibold text-[#EAF3EE]"
            >
              <svg viewBox="0 0 24 24" fill="none" className="h-5 w-5">
                <path
                  d="M12 3v7M12 10L6 14v4M12 10l6 4v4"
                  stroke="#34D399"
                  strokeWidth={1.7}
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
                <circle cx="12" cy="3" r="1.6" fill="#34D399" />
                <circle cx="6" cy="18" r="1.6" fill="#F5B942" />
                <circle cx="18" cy="18" r="1.6" fill="#F5B942" />
              </svg>
              cascade
            </a>
            <p className="mt-3.5 max-w-[260px] text-[13.5px] leading-relaxed text-[#7C9188]">
              Open-source funding infrastructure for the Stellar ecosystem. Splits, streams, and
              dependency cascading, built on Soroban.
            </p>
          </div>

          {columns.map((col) => (
            <div key={col.title}>
              <h4 className="mb-4 font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.08em] text-[#7C9188]">
                {col.title}
              </h4>
              {col.links.map((link) => (
                <a
                  key={link.label}
                  href={link.href}
                  className="mb-2.5 block text-sm text-[#EAF3EE] opacity-85 transition-opacity hover:text-[#34D399] hover:opacity-100"
                >
                  {link.label}
                </a>
              ))}
            </div>
          ))}
        </div>

        <div className="flex flex-wrap items-center justify-between gap-4 border-t border-[#1E2E27] pt-7 font-['IBM_Plex_Mono'] text-[12.5px] text-[#7C9188]">
          <span>CASCADE — OPEN SOURCE, STELLAR/SOROBAN NATIVE</span>
          <span>BUILT IN THE OPEN</span>
        </div>
      </div>
    </footer>
  );
}
