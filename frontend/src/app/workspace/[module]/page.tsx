import WorkspaceModule from "@/components/WorkspaceModule";
export default async function Page({
  params,
}: {
  params: Promise<{ module: string }>;
}) {
  const { module } = await params;
  return <WorkspaceModule module={module} />;
}
