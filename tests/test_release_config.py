"""Guard against accidental deployments, paid plans and public sandbox exposure."""
import json
import pathlib
import unittest
import yaml

ROOT=pathlib.Path(__file__).parents[1]
class ReleaseConfiguration(unittest.TestCase):
    def test_git_deployments_remain_disabled(self):
        for file in ['vercel.json','frontend/vercel.json']:
            self.assertIs(json.loads((ROOT/file).read_text())['git']['deploymentEnabled'],False)
    def test_single_free_backend_no_automatic_deployment(self):
        config=yaml.safe_load((ROOT/'render.yaml').read_text())
        self.assertEqual(config['previews']['generation'],'off')
        self.assertEqual(len(config['services']),1)
        service=config['services'][0]
        self.assertEqual(service['plan'],'free')
        self.assertEqual(service['autoDeployTrigger'],'off')
        self.assertEqual(service['healthCheckPath'],'/ready')
        env={v['key']:v.get('value') for v in service['envVars']}
        for name in ['LLM_ENABLED','ENABLE_SANDBOX','BACKGROUND_PROCESSING']:self.assertEqual(env[name],'false')
        self.assertEqual(env['BETA_MODE'],'true')
        self.assertEqual(env['COOKIE_SECURE'],'true')
    def test_no_deploy_commands_in_ci(self):
        config=yaml.safe_load((ROOT/'.github/workflows/ci.yml').read_text())
        commands='\n'.join(str(step.get('run','')) for job in config['jobs'].values() for step in job['steps'])
        for forbidden in ['vercel deploy','vercel --prod','render deploy','terraform apply','git push']:self.assertNotIn(forbidden,commands)
if __name__=='__main__':unittest.main()
