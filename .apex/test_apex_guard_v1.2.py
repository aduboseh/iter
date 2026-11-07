#!/usr/bin/env python3
"""Synthetic guard tests. These do not prove model compliance."""
from pathlib import Path
import os,subprocess,tempfile,unittest
SCRIPT=Path(__file__).with_name("apex-guard-v1.2.sh")
def run(changes,basefiles=None,env=None):
 with tempfile.TemporaryDirectory() as t:
  p=Path(t)
  def git(*a):
   r=subprocess.run(["git",*a],cwd=p,capture_output=True,text=True)
   if r.returncode:raise RuntimeError(r.stderr)
  git("init","-q");git("config","user.email","apex@example.invalid");git("config","user.name","APEX")
  files={"AGENTS.md":"# instructions\n",".apex/protected-paths.txt":"governance/**\n","src/a.py":"assert value == 1\n"}
  files.update(basefiles or {})
  for name,s in files.items():
   f=p/name;f.parent.mkdir(parents=True,exist_ok=True);f.write_text(s)
  git("add",".");git("commit","-qm","baseline");git("branch","base");git("checkout","-qb","candidate")
  for name,s in changes.items():
   f=p/name
   if s is None:f.unlink(missing_ok=True)
   else:f.parent.mkdir(parents=True,exist_ok=True);f.write_text(s)
  git("add","-A");git("commit","--allow-empty","-qm","candidate")
  e=os.environ.copy();e["APEX_GLOBAL_BYTES"]="0";e.update(env or {})
  r=subprocess.run(["bash",str(SCRIPT),"base"],cwd=p,env=e,capture_output=True,text=True)
  return r.returncode,r.stdout+r.stderr
class Tests(unittest.TestCase):
 def test_clean(self):self.assertEqual(run({"src/a.py":"assert value == 2\n"})[0],0)
 def test_protected(self):self.assertEqual(run({"governance/policy.md":"x\n"})[0],1)
 def test_ci(self):self.assertEqual(run({".github/workflows/x.yml":"jobs: {}\n"})[0],1)
 def test_skip(self):self.assertEqual(run({"tests/test_x.py":"@pytest.mark.skip\n"})[0],1)
 def test_tautology(self):self.assertEqual(run({"src/a.py":"assert True\n"})[0],1)
 def test_delete(self):self.assertEqual(run({"tests/test_x.py":None},{"tests/test_x.py":"assert 1\n"})[0],1)
 def test_no_manifest(self):self.assertEqual(run({".apex/protected-paths.txt":None})[0],2)
 def test_budget(self):self.assertEqual(run({},env={"APEX_GLOBAL_BYTES":"50000"})[0],1)
 def test_bypass(self):self.assertEqual(run({".github/workflows/x.yml":"jobs: {}\n"},env={"APEX_PROTECTED_OK":"1"})[0],1)
 def test_docs(self):self.assertEqual(run({"README.md":"#[ignore] is discussed\n"})[0],0)
if __name__=="__main__":unittest.main(verbosity=2)
