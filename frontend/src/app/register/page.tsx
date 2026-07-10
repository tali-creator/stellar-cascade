import CascadeRegisterForm from '@/components/register/CascadeRegisterForm';

export const metadata = {
  title: 'Register a project — Cascade',
  description: 'Register your project and declare its dependency splits on-chain.',
};

export default function RegisterPage() {
  return (
    <main className="min-h-screen flex flex-col" style={{ background: '#0A1410' }}>
      <CascadeRegisterForm />
    </main>
  );
}
