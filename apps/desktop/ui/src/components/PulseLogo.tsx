export function PulseLogo({ size = 16 }: { size?: number }) {
  return (
    <svg
      width={size * 0.56}
      height={size * 0.56}
      viewBox="0 0 16 16"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden
    >
      <path
        d="M1 8 L4 8 L5.2 4 L7 12 L9 6 L10.5 9 L12 8 L15 8"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
      />
    </svg>
  );
}

export function PulseLogoMark({ size = 22 }: { size?: number }) {
  return (
    <span
      style={{
        width: size,
        height: size,
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        borderRadius: 6,
        background: 'linear-gradient(135deg, var(--pulse), var(--pulse2))',
        boxShadow: '0 0 12px var(--pulse-g)',
        color: '#fff',
      }}
    >
      <PulseLogo size={size} />
    </span>
  );
}
