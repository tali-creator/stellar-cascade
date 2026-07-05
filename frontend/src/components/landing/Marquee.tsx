import "./animations.css";

const items = [
  { value: "~5s", label: "finality on settlement" },
  { value: "200", label: "recipients per list" },
  { value: "USDC / EURC", label: "supported assets" },
  { value: "Permissionless", label: "split triggering" },
  { value: "No gas", label: "for claiming maintainers" },
];

export default function Marquee() {
  // duplicated once so the animation can loop seamlessly
  const track = [...items, ...items];

  return (
    <div className="overflow-hidden border-y border-[#1E2E27] bg-[#0F1D17] py-4">
      <div className="marquee-track flex w-max gap-14 whitespace-nowrap">
        {track.map((item, i) => (
          <span
            key={i}
            className="flex items-center gap-2.5 font-['IBM_Plex_Mono'] text-sm text-[#7C9188]"
          >
            <b className="font-semibold text-[#34D399]">{item.value}</b> {item.label}
          </span>
        ))}
      </div>
    </div>
  );
}
