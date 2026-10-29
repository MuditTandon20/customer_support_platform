"""Ensure scanner ambiguity cannot silently become a release clearance."""
import pathlib
import sys
import unittest
sys.path.insert(0,str(pathlib.Path(__file__).parents[1]/'scripts'))
from review_security import assess_finding

class AssessmentGate(unittest.TestCase):
    def setUp(self):
        self.row={'package':'stdlib','id':'CVE-synthetic','target':'usr/local/bin/gosu'}
        self.advisories={'CVE-synthetic':{'id':'GO-synthetic','affected':[{'ecosystem_specific':{'imports':[{'path':'crypto/tls','symbols':['Conn.Handshake']}]}}]}}
        self.findings=[{'osv':'GO-synthetic','trace':[{'module':'stdlib','version':'v1.24.6'}]}]
    def test_unknown_or_unscoped_advisory_remains_on_hold(self):
        self.assertIsNone(assess_finding(self.row,'local_postgres',{},self.findings))
        self.assertIsNone(assess_finding(self.row,'local_postgres',{'CVE-synthetic':{'id':'GO-synthetic','affected':[]}},self.findings))
        self.assertIsNone(assess_finding({'package':'other','id':'CVE-new'},'api',{},[]))
    def test_linked_vulnerable_symbol_remains_on_hold(self):
        linked=self.findings+[{'osv':'GO-synthetic','trace':[{'module':'stdlib','package':'crypto/tls','function':'Handshake'}]}]
        self.assertIsNone(assess_finding(self.row,'local_postgres',self.advisories,linked))
    def test_binary_and_product_evidence_cannot_be_reused_for_another_target(self):
        self.assertIsNotNone(assess_finding(self.row,'local_postgres',self.advisories,self.findings))
        self.assertIsNone(assess_finding(self.row|{'target':'usr/bin/another-go-binary'},'local_postgres',self.advisories,self.findings))
        self.assertIsNone(assess_finding(self.row,'api',self.advisories,self.findings))
        self.assertIsNone(assess_finding({'package':'libxml2','id':'CVE-2026-6653'},'ai',{},[]))
    def test_imported_package_without_function_scope_is_not_cleared(self):
        advisory={'CVE-synthetic':{'id':'GO-synthetic','affected':[{'ecosystem_specific':{'imports':[{'path':'os'}]}}]}}
        imported=[{'osv':'GO-synthetic','trace':[{'module':'stdlib','package':'os'}]}]
        self.assertIsNone(assess_finding(self.row,'local_postgres',advisory,imported))

if __name__=='__main__':unittest.main()
