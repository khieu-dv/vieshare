// src/app/frps/page.tsx
import { Header } from '~/ui/components/header';
import FrpsManager from '../components/FrpsManager';

export default function FrpsPage() {
    return (
        <div className="min-h-screen bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100">
            <Header />
            <section className="py-10"></section>
            <FrpsManager />
        </div>
    );
}


