export default function Header() {
  return (
    <header className="fixed top-0 left-0 right-0 z-50 border-b border-[#1E2E27] bg-[#0A1410]/85 backdrop-blur-md">
      <nav className="mx-auto flex h-[72px] max-w-[1180px] items-center justify-between px-8">
        <a
          href="#top"
          className="flex items-center gap-2.5 font-['IBM_Plex_Mono'] text-lg font-semibold tracking-tight text-[#EAF3EE]"
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

        <div className="hidden items-center gap-9 md:flex">
          <a href="#how" className="text-sm text-[#7C9188] transition-colors hover:text-[#EAF3EE]">
            How it works
          </a>
          <a href="#modes" className="text-sm text-[#7C9188] transition-colors hover:text-[#EAF3EE]">
            Streams &amp; gives
          </a>
          <a href="#compare" className="text-sm text-[#7C9188] transition-colors hover:text-[#EAF3EE]">
            Why Soroban
          </a>
          <a href="#start" className="text-sm text-[#7C9188] transition-colors hover:text-[#EAF3EE]">
            Get started
          </a>
        </div>

        <a
          href="#start"
          className="rounded-[3px] border border-[#34D399] px-[18px] py-[9px] font-['IBM_Plex_Mono'] text-[13px] font-semibold text-[#34D399] transition-colors hover:bg-[#34D399] hover:text-[#0A1410]"
        >
          Claim your repo →
        </a>
      </nav>
    </header>
  );
}
